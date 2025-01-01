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

use crate::domain::{
    models::{DomainError, Store},
    ports::outbound::QueuePort,
};

const MESSAGE_TTL: i32 = 300000; // 5 minutes in milliseconds
const TASK_EXPIRY: i64 = 300; // 5 minutes in seconds

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub query: String,
    pub store: Option<Store>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
pub struct RabbitMQAdapter {
    connection: Arc<Connection>,
    channel: Arc<Channel>,
    redis: ConnectionManager,
}

impl RabbitMQAdapter {
    fn get_task_key(task_id: &str) -> String {
        format!("task:{}", task_id)
    }

    fn get_queue_name(store: &Store) -> String {
        format!("scraper_tasks_{}", store.to_string().to_lowercase())
    }

    pub async fn new(amqp_url: String, redis_url: String) -> Result<Self, DomainError> {
        // Initialize RabbitMQ connection
        info!("Connecting to RabbitMQ at: {}", amqp_url);
        let connection = Arc::new(Connection::connect(
            &amqp_url,
            ConnectionProperties::default()
                .with_connection_name("scraper-service".into()),
        ).await.map_err(|e| DomainError::queue(format!("Failed to connect to RabbitMQ: {}", e)))?);
        
        let channel = Arc::new(connection.create_channel().await
            .map_err(|e| DomainError::queue(format!("Failed to create channel: {}", e)))?);
        
        // Initialize Redis connection
        let redis = redis::Client::open(redis_url)
            .map_err(|e| DomainError::queue(format!("Failed to create Redis client: {}", e)))?
            .get_tokio_connection_manager()
            .await
            .map_err(|e| DomainError::queue(format!("Failed to get Redis connection: {}", e)))?;
        
        Ok(Self {
            connection,
            channel,
            redis,
        })
    }

    async fn ensure_queue_exists(&self, store: &Store) -> Result<(), DomainError> {
        let queue_name = Self::get_queue_name(store);
        
        // Try to declare the queue passively first (check if it exists)
        match self.channel.queue_declare(
            &queue_name,
            QueueDeclareOptions {
                passive: true,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        ).await {
            Ok(_) => {
                info!("Queue {} already exists", queue_name);
                return Ok(());
            },
            Err(_) => {
                info!("Queue {} does not exist, creating it", queue_name);
            }
        }
        
        // Declare the dead letter exchange if needed
        match self.channel.exchange_declare(
            "dl.scraper_tasks",
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions {
                passive: true,
                ..ExchangeDeclareOptions::default()
            },
            FieldTable::default(),
        ).await {
            Ok(_) => {
                info!("Dead letter exchange already exists");
            },
            Err(_) => {
                info!("Creating dead letter exchange");
                self.channel.exchange_declare(
                    "dl.scraper_tasks",
                    lapin::ExchangeKind::Direct,
                    ExchangeDeclareOptions::default(),
                    FieldTable::default(),
                ).await.map_err(|e| DomainError::queue(format!("Failed to declare dead letter exchange: {}", e)))?;
            }
        }
        
        // Declare the dead letter queue if needed
        match self.channel.queue_declare(
            "dl.scraper_tasks",
            QueueDeclareOptions {
                passive: true,
                ..QueueDeclareOptions::default()
            },
            FieldTable::default(),
        ).await {
            Ok(_) => {
                info!("Dead letter queue already exists");
            },
            Err(_) => {
                info!("Creating dead letter queue");
                self.channel.queue_declare(
                    "dl.scraper_tasks",
                    QueueDeclareOptions {
                        durable: true,
                        ..QueueDeclareOptions::default()
                    },
                    FieldTable::default(),
                ).await.map_err(|e| DomainError::queue(format!("Failed to declare dead letter queue: {}", e)))?;
                
                // Bind the dead letter queue to the exchange
                self.channel.queue_bind(
                    "dl.scraper_tasks",
                    "dl.scraper_tasks",
                    "",
                    QueueBindOptions::default(),
                    FieldTable::default(),
                ).await.map_err(|e| DomainError::queue(format!("Failed to bind dead letter queue: {}", e)))?;
            }
        }
        
        // Declare the queue with dead letter exchange
        let mut args = FieldTable::default();
        args.insert("x-message-ttl".into(), AMQPValue::LongInt(MESSAGE_TTL));
        args.insert("x-dead-letter-exchange".into(), AMQPValue::LongString("dl.scraper_tasks".into()));
        
        // Now declare the queue with all required properties
        self.channel.queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                exclusive: false,
                passive: false,
                ..QueueDeclareOptions::default()
            },
            args,
        ).await.map_err(|e| DomainError::queue(format!("Failed to declare queue {}: {}", queue_name, e)))?;
        
        info!("Queue {} is ready", queue_name);
        Ok(())
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

    async fn create_and_store_task(&self, query: &str, store: Option<&Store>) -> Result<(), DomainError> {
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            query: query.to_string(),
            store: store.cloned(),
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
        
        match store {
            Some(store) => {
                // Ensure queue exists and publish to store-specific queue
                self.ensure_queue_exists(store).await?;
                let queue_name = Self::get_queue_name(store);
                
                info!("Publishing task to queue {}: {}", queue_name, query);
                
                self.channel.basic_publish(
                    "",
                    &queue_name,
                    BasicPublishOptions::default(),
                    query.as_bytes(),
                    BasicProperties::default()
                        .with_expiration(MESSAGE_TTL.to_string().into()),
                ).await.map_err(|e| DomainError::queue(format!("Failed to publish task: {}", e)))?;
            },
            None => {
                // For all-stores case, publish to each store's queue
                for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
                    self.ensure_queue_exists(store).await?;
                    let queue_name = Self::get_queue_name(store);
                    
                    info!("Publishing task to queue {}: {}", queue_name, query);
                    
                    self.channel.basic_publish(
                        "",
                        &queue_name,
                        BasicPublishOptions::default(),
                        query.as_bytes(),
                        BasicProperties::default()
                            .with_expiration(MESSAGE_TTL.to_string().into()),
                    ).await.map_err(|e| DomainError::queue(format!("Failed to publish task: {}", e)))?;
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
        
        // For all-stores job, enqueue to each store's queue
        for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            self.ensure_queue_exists(store).await?;
            let queue_name = Self::get_queue_name(store);
            
            self.channel.basic_publish(
                "",
                &queue_name,
                BasicPublishOptions::default(),
                query.as_bytes(),
                BasicProperties::default()
                    .with_expiration(MESSAGE_TTL.to_string().into()),
            ).await.map_err(|e| DomainError::queue(format!("Failed to publish task: {}", e)))?;
        }
        
        Ok(())
    }

    async fn enqueue_store_scrape_job(&self, query: &str, store: &Store) -> Result<(), DomainError> {
        // Clean up old tasks first
        if let Err(e) = self.cleanup_old_tasks().await {
            error!("Failed to cleanup old tasks: {}", e);
        }
        
        // Ensure queue exists and publish to store-specific queue
        self.ensure_queue_exists(store).await?;
        let queue_name = Self::get_queue_name(store);
        
        self.channel.basic_publish(
            "",
            &queue_name,
            BasicPublishOptions::default(),
            query.as_bytes(),
            BasicProperties::default()
                .with_expiration(MESSAGE_TTL.to_string().into()),
        ).await.map_err(|e| DomainError::queue(format!("Failed to publish task: {}", e)))?;
        
        Ok(())
    }
    
    async fn process_scrape_job(&self, query: &str) -> Result<(), DomainError> {
        // Process jobs from all store queues
        for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            let queue_name = Self::get_queue_name(store);
            
            if let Some(delivery) = self.channel.basic_get(
                &queue_name,
                BasicGetOptions::default(),
            ).await.map_err(|e| DomainError::queue(format!("Failed to get task from queue {}: {}", queue_name, e)))? {
                let query_str = String::from_utf8(delivery.data.clone())
                    .map_err(|e| DomainError::queue(format!("Failed to parse query data: {}", e)))?;
                
                info!("Processing task for store: {} and query: {}", store, query_str);
                
                // Acknowledge the message
                delivery.ack(BasicAckOptions::default()).await
                    .map_err(|e| DomainError::queue(format!("Failed to acknowledge task: {}", e)))?;
            }
        }
        
        Ok(())
    }

    async fn process_store_scrape_job(&self, query: &str, store: &Store) -> Result<(), DomainError> {
        let queue_name = Self::get_queue_name(store);
        
        if let Some(delivery) = self.channel.basic_get(
            &queue_name,
            BasicGetOptions::default(),
        ).await.map_err(|e| DomainError::queue(format!("Failed to get task from queue {}: {}", queue_name, e)))? {
            let query_str = String::from_utf8(delivery.data.clone())
                .map_err(|e| DomainError::queue(format!("Failed to parse query data: {}", e)))?;
            
            info!("Processing task for store: {} and query: {}", store, query_str);
            
            // Acknowledge the message
            delivery.ack(BasicAckOptions::default()).await
                .map_err(|e| DomainError::queue(format!("Failed to acknowledge task: {}", e)))?;
        }
        
        Ok(())
    }
} 