use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use lapin::{
    options::*, types::{FieldTable, AMQPValue}, BasicProperties,
    Connection, ConnectionProperties, Channel,
};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Serialize, Deserialize};
use tracing::{info, error};
use uuid::Uuid;

use crate::domain::models::DomainError;
use crate::domain::ports::outbound::QueuePort;

const MESSAGE_TTL: i32 = 300000; // 5 minutes in milliseconds
const TASK_EXPIRY: i64 = 300; // 5 minutes in seconds

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub query: String,
    pub store: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
pub struct RabbitMQAdapter {
    connection: Arc<Connection>,
    channel: Arc<Channel>,
    queue_name: String,
    redis: ConnectionManager,
}

impl RabbitMQAdapter {
    fn get_task_key(task_id: &str) -> String {
        format!("task:{}", task_id)
    }

    pub async fn new(amqp_addr: String, redis_url: String) -> Result<Self, DomainError> {
        // Initialize RabbitMQ connection
        info!("Connecting to RabbitMQ at: {}", amqp_addr);
        let connection = Arc::new(Connection::connect(
            &amqp_addr,
            ConnectionProperties::default()
                .with_connection_name("scraper-service".into()),
        ).await.map_err(|e| DomainError::queue(format!("Failed to connect to RabbitMQ: {}", e)))?);
        
        let channel = Arc::new(connection.create_channel().await
            .map_err(|e| DomainError::queue(format!("Failed to create channel: {}", e)))?);
        let queue_name = "scraper_tasks".to_string();
        
        // Declare the dead letter exchange
        info!("Setting up dead letter exchange...");
        channel.exchange_declare(
            "dl.scraper_tasks",
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        ).await.map_err(|e| DomainError::queue(format!("Failed to declare dead letter exchange: {}", e)))?;
        
        // Declare the dead letter queue
        info!("Setting up dead letter queue...");
        channel.queue_declare(
            "dl.scraper_tasks",
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        ).await.map_err(|e| DomainError::queue(format!("Failed to declare dead letter queue: {}", e)))?;
        
        // Bind the dead letter queue to the exchange
        channel.queue_bind(
            "dl.scraper_tasks",
            "dl.scraper_tasks",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        ).await.map_err(|e| DomainError::queue(format!("Failed to bind dead letter queue: {}", e)))?;
        
        // Create arguments for the main queue with TTL and dead letter config
        info!("Setting up main queue with TTL...");
        let mut args = FieldTable::default();
        args.insert("x-message-ttl".into(), AMQPValue::LongInt(MESSAGE_TTL));
        args.insert("x-dead-letter-exchange".into(), AMQPValue::LongString("dl.scraper_tasks".into()));
        
        // Declare the main queue with the arguments
        channel.queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
            args,
        ).await.map_err(|e| DomainError::queue(format!("Failed to declare main queue: {}", e)))?;
        
        info!("RabbitMQ setup completed successfully");
        
        // Initialize Redis connection
        let redis = redis::Client::open(redis_url)
            .map_err(|e| DomainError::queue(format!("Failed to create Redis client: {}", e)))?
            .get_tokio_connection_manager()
            .await
            .map_err(|e| DomainError::queue(format!("Failed to get Redis connection: {}", e)))?;
        
        Ok(Self {
            connection,
            channel,
            queue_name,
            redis,
        })
    }

    async fn cleanup_old_tasks(&self) -> Result<(), DomainError> {
        let mut conn = self.redis.clone();
        let now = Utc::now().timestamp();
        
        // Get all task keys
        let keys: Vec<String> = conn.keys("task:*").await
            .map_err(|e| DomainError::queue(format!("Failed to get task keys: {}", e)))?;
        
        // Check each task and remove if expired
        for key in keys {
            let task_json: Option<String> = conn.get(&key).await
                .map_err(|e| DomainError::queue(format!("Failed to get task: {}", e)))?;
            
            if let Some(json) = task_json {
                if let Ok(task) = serde_json::from_str::<Task>(&json) {
                    if (now - task.created_at.timestamp()) > TASK_EXPIRY {
                        // Remove expired task
                        let _: () = conn.del(&key).await
                            .map_err(|e| DomainError::queue(format!("Failed to delete task: {}", e)))?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl QueuePort for RabbitMQAdapter {
    async fn enqueue_scrape_job(&self, query: &str) -> Result<(), DomainError> {
        // Clean up old tasks first
        if let Err(e) = self.cleanup_old_tasks().await {
            error!("Failed to cleanup old tasks: {}", e);
        }
        
        // Parse store and query from the input
        let parts: Vec<&str> = query.split(':').collect();
        if parts.len() != 2 {
            return Err(DomainError::queue(format!("Invalid task format: {}", query)));
        }
        
        let store = parts[0];
        let query = parts[1];
        
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            query: query.to_string(),
            store: store.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Store task in Redis with expiration
        let mut conn = self.redis.clone();
        let task_key = Self::get_task_key(&task_id);
        let task_json = serde_json::to_string(&task)
            .map_err(|e| DomainError::queue(format!("Failed to serialize task: {}", e)))?;
        let _: () = conn.set_ex(&task_key, task_json, TASK_EXPIRY as usize).await
            .map_err(|e| DomainError::queue(format!("Failed to store task in Redis: {}", e)))?;
        
        // Publish to RabbitMQ with TTL
        self.channel.basic_publish(
            "",
            &self.queue_name,
            BasicPublishOptions::default(),
            format!("{}:{}", store, query).as_bytes(),
            BasicProperties::default()
                .with_expiration(MESSAGE_TTL.to_string().into()),
        ).await.map_err(|e| DomainError::queue(format!("Failed to publish task: {}", e)))?;
        
        Ok(())
    }
    
    async fn process_scrape_job(&self, query: &str) -> Result<(), DomainError> {
        if let Some(delivery) = self.channel.basic_get(
            &self.queue_name,
            BasicGetOptions::default(),
        ).await.map_err(|e| DomainError::queue(format!("Failed to get task from queue: {}", e)))? {
            let task_str = String::from_utf8(delivery.data.clone())
                .map_err(|e| DomainError::queue(format!("Failed to parse task data: {}", e)))?;
            
            // Parse store and query from task
            let parts: Vec<&str> = task_str.split(':').collect();
            if parts.len() != 2 {
                return Err(DomainError::queue(format!("Invalid task format: {}", task_str)));
            }
            
            let store = parts[0];
            let query = parts[1];
            
            info!("Processing task for store: {} and query: {}", store, query);
            
            // Acknowledge the message
            delivery.ack(BasicAckOptions::default()).await
                .map_err(|e| DomainError::queue(format!("Failed to acknowledge task: {}", e)))?;
        }
        
        Ok(())
    }
} 