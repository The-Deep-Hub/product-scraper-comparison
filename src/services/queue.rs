use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use lapin::{
    options::*, types::FieldTable, BasicProperties,
    Connection, ConnectionProperties, Channel,
};
use tracing::{info, error};
use redis::{aio::ConnectionManager, AsyncCommands};
use std::time::Duration;
use std::fmt;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Processing => write!(f, "processing"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Processing => "processing",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "processing" => Some(TaskStatus::Processing),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub query: String,
    pub status: TaskStatus,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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
    redis: ConnectionManager,
}

impl RabbitMQQueue {
    pub async fn new() -> AppResult<Self> {
        // Initialize RabbitMQ connection
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

        // Initialize Redis connection
        let redis_url = format!(
            "redis://{}:{}@{}:{}/",
            std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
            std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
            std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
        );
        let redis_client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(redis_client).await?;
        
        Ok(Self {
            connection,
            channel,
            queue_name,
            redis,
        })
    }

    fn get_task_key(task_id: &str) -> String {
        format!("task:{}", task_id)
    }
}

#[async_trait]
impl QueueService for RabbitMQQueue {
    async fn create_task(&self, query: String) -> AppResult<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let task = Task {
            id: task_id.clone(),
            query,
            status: TaskStatus::Pending,
            error: None,
            created_at: now,
            updated_at: now,
        };
        
        // Store task in Redis
        let mut redis = self.redis.clone();
        let task_key = Self::get_task_key(&task_id);
        let task_json = serde_json::to_string(&task)?;
        redis.set_ex(&task_key, task_json, 3600).await?; // 1 hour TTL
        
        // Publish task to RabbitMQ
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
    
    async fn get_task(&self, task_id: &str) -> AppResult<Task> {
        let mut redis = self.redis.clone();
        let task_key = Self::get_task_key(task_id);
        
        let task_json: Option<String> = redis.get(&task_key).await?;
        match task_json {
            Some(json) => {
                let task: Task = serde_json::from_str(&json)?;
                Ok(task)
            }
            None => Err(AppError::NotFound(format!("Task {} not found", task_id)))
        }
    }
    
    async fn get_pending_task(&self) -> AppResult<Option<Task>> {
        if let Some(delivery) = self.channel.basic_get(
            &self.queue_name,
            BasicGetOptions::default(),
        ).await? {
            let task: Task = serde_json::from_slice(&delivery.data)?;
            
            // Update task status to Processing
            let mut updated_task = task.clone();
            updated_task.status = TaskStatus::Processing;
            updated_task.updated_at = chrono::Utc::now();
            
            let mut redis = self.redis.clone();
            let task_key = Self::get_task_key(&task.id);
            let task_json = serde_json::to_string(&updated_task)?;
            redis.set_ex(&task_key, task_json, 3600).await?;
            
            self.channel.basic_ack(
                delivery.delivery_tag,
                BasicAckOptions::default(),
            ).await?;
            
            Ok(Some(updated_task))
        } else {
            Ok(None)
        }
    }
    
    async fn mark_task_completed(&self, task_id: String) -> AppResult<()> {
        let mut redis = self.redis.clone();
        let task_key = Self::get_task_key(&task_id);
        
        // Get current task
        let task_json: Option<String> = redis.get(&task_key).await?;
        if let Some(json) = task_json {
            let mut task: Task = serde_json::from_str(&json)?;
            task.status = TaskStatus::Completed;
            task.updated_at = chrono::Utc::now();
            
            // Update task in Redis
            let updated_json = serde_json::to_string(&task)?;
            redis.set_ex(&task_key, updated_json, 3600).await?;
            
            info!("Task completed: {}", task_id);
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Task {} not found", task_id)))
        }
    }
    
    async fn mark_task_failed(&self, task_id: String, error: String) -> AppResult<()> {
        let mut redis = self.redis.clone();
        let task_key = Self::get_task_key(&task_id);
        
        // Get current task
        let task_json: Option<String> = redis.get(&task_key).await?;
        if let Some(json) = task_json {
            let mut task: Task = serde_json::from_str(&json)?;
            task.status = TaskStatus::Failed;
            task.error = Some(error.clone());
            task.updated_at = chrono::Utc::now();
            
            // Update task in Redis
            let updated_json = serde_json::to_string(&task)?;
            redis.set_ex(&task_key, updated_json, 3600).await?;
            
            error!("Task failed: {} - {}", task_id, error);
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Task {} not found", task_id)))
        }
    }
} 