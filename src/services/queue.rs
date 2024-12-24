use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use lapin::{
    options::*, types::FieldTable, BasicProperties,
    Connection, ConnectionProperties, Channel,
};
use tracing::{info, error};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub query: String,
    pub status: String,
    pub error: Option<String>,
}

#[async_trait]
pub trait QueueService: Send + Sync {
    async fn create_task(&self, query: String) -> AppResult<String>;
    async fn get_task(&self, task_id: &str) -> AppResult<Task>;
    async fn get_pending_task(&self) -> AppResult<Option<Task>>;
    async fn mark_task_completed(&self, task_id: String) -> AppResult<()>;
    async fn mark_task_failed(&self, task_id: String, error: String) -> AppResult<()>;
}

pub struct RabbitMQQueue {
    connection: Connection,
    channel: Channel,
    queue_name: String,
}

impl RabbitMQQueue {
    pub async fn new() -> AppResult<Self> {
        let addr = std::env::var("AMQP_ADDR")
            .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string());
        
        info!("Connecting to RabbitMQ at: {}", addr);
        let connection = Connection::connect(
            &addr,
            ConnectionProperties::default()
                .with_connection_name("scraper-service".into()),
        ).await?;
        
        let channel = connection.create_channel().await?;
        let queue_name = "scraper_tasks".to_string();
        
        channel.queue_declare(
            &queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).await?;
        
        Ok(Self {
            connection,
            channel,
            queue_name,
        })
    }
}

#[async_trait]
impl QueueService for RabbitMQQueue {
    async fn create_task(&self, query: String) -> AppResult<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            query,
            status: "pending".to_string(),
            error: None,
        };
        
        let payload = serde_json::to_vec(&task)?;
        
        self.channel.basic_publish(
            "",
            &self.queue_name,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        ).await?;
        
        info!("Created task: {}", task_id);
        Ok(task_id)
    }
    
    async fn get_task(&self, _task_id: &str) -> AppResult<Task> {
        // This is a placeholder - implement actual task retrieval from a persistent store
        Err(AppError::BadRequest("Task retrieval not implemented".into()))
    }
    
    async fn get_pending_task(&self) -> AppResult<Option<Task>> {
        if let Some(delivery) = self.channel.basic_get(
            &self.queue_name,
            BasicGetOptions::default(),
        ).await? {
            let task: Task = serde_json::from_slice(&delivery.data)?;
            self.channel.basic_ack(
                delivery.delivery_tag,
                BasicAckOptions::default(),
            ).await?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
    
    async fn mark_task_completed(&self, task_id: String) -> AppResult<()> {
        // This is a placeholder - implement actual task status update in a persistent store
        info!("Task completed: {}", task_id);
        Ok(())
    }
    
    async fn mark_task_failed(&self, task_id: String, error: String) -> AppResult<()> {
        // This is a placeholder - implement actual task status update in a persistent store
        error!("Task failed: {} - {}", task_id, error);
        Ok(())
    }
} 