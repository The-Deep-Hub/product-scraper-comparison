use async_trait::async_trait;
use lapin::{
    options::*, types::FieldTable, BasicProperties,
    Connection, ConnectionProperties, Channel,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;
use redis::AsyncCommands;

use crate::{
    error::AppError,
    models::product::Product,
    services::queue::{QueueService, Task, TaskStatus, ResponseSender},
};

pub struct RabbitMQQueue {
    channel: Channel,
    redis: redis::Client,
    response_channels: Arc<Mutex<HashMap<String, ResponseSender>>>,
}

impl RabbitMQQueue {
    pub async fn new() -> Result<Self, AppError> {
        let redis_url = format!(
            "redis://{}:{}@{}:{}/",
            std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
            std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
            std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
        );
        
        let rabbitmq_url = format!(
            "amqp://{}:{}@{}:{}",
            std::env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string()),
            std::env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string()),
            std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string()),
            std::env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string()),
        );

        tracing::info!("Connecting to RabbitMQ at: {}", rabbitmq_url);
        let conn = Connection::connect(
            &rabbitmq_url,
            ConnectionProperties::default(),
        ).await.map_err(|e| AppError::QueueError(format!("Failed to connect to RabbitMQ: {}", e)))?;

        let channel = conn.create_channel().await
            .map_err(|e| AppError::QueueError(format!("Failed to create channel: {}", e)))?;

        channel.queue_declare(
            "scraper_tasks",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).await.map_err(|e| AppError::QueueError(format!("Failed to declare queue: {}", e)))?;

        tracing::info!("Connecting to Redis at: {}", redis_url.replace(|c: char| c.is_ascii_alphanumeric(), "*"));
        let redis = redis::Client::open(redis_url)
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            channel,
            redis,
            response_channels: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl QueueService for RabbitMQQueue {
    async fn create_task(&self, query: String) -> Result<String, AppError> {
        let task_id = Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            query,
            status: TaskStatus::Pending,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let task_json = serde_json::to_string(&task)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        tracing::info!("Creating task {} for query: {}", task.id, task.query);
        
        // Store task in Redis
        tracing::info!("Storing task in Redis with key: task:{}", task.id);
        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;
        redis_conn.set_ex(format!("task:{}", task.id), task_json.clone(), 3600).await
            .map_err(|e| AppError::CacheError(format!("Failed to store task in Redis: {}", e)))?;

        // Publish task to RabbitMQ
        tracing::info!("Publishing task {} to RabbitMQ queue: scraper_tasks", task.id);
        self.channel.basic_publish(
            "",
            "scraper_tasks",
            BasicPublishOptions::default(),
            task_json.as_bytes(),
            BasicProperties::default(),
        ).await.map_err(|e| AppError::QueueError(format!("Failed to publish task: {}", e)))?;

        tracing::info!("Task {} created and published successfully", task.id);
        Ok(task_id)
    }

    async fn create_task_with_response(&self, query: String, response_tx: ResponseSender) -> Result<String, AppError> {
        let task_id = self.create_task(query).await?;
        
        // Store the response channel
        self.response_channels.lock().await.insert(task_id.clone(), response_tx);
        
        Ok(task_id)
    }

    async fn get_task(&self, task_id: &str) -> Result<Task, AppError> {
        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let task_json: String = redis_conn.get(format!("task:{}", task_id)).await
            .map_err(|e| AppError::CacheError(format!("Failed to get task from Redis: {}", e)))?;

        serde_json::from_str(&task_json)
            .map_err(|e| AppError::SerializationError(format!("Failed to deserialize task: {}", e)))
    }

    async fn get_pending_task(&self) -> Result<Option<Task>, AppError> {
        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let result = self.channel.basic_get(
            "scraper_tasks",
            BasicGetOptions { no_ack: true, ..Default::default() },
        ).await.map_err(|e| AppError::QueueError(format!("Failed to get task from queue: {}", e)))?;

        if let Some(delivery) = result {
            let task: Task = serde_json::from_slice(&delivery.data)
                .map_err(|e| AppError::SerializationError(format!("Failed to deserialize task: {}", e)))?;

            // Mark task as processing
            self.mark_task_processing(task.id.clone()).await?;

            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    async fn mark_task_processing(&self, task_id: String) -> Result<(), AppError> {
        let mut task = self.get_task(&task_id).await?;
        task.status = TaskStatus::Processing;
        task.updated_at = Utc::now();

        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let task_json = serde_json::to_string(&task)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        redis_conn.set_ex(format!("task:{}", task_id), task_json, 3600).await
            .map_err(|e| AppError::CacheError(format!("Failed to update task in Redis: {}", e)))?;

        Ok(())
    }

    async fn mark_task_completed(&self, task_id: String) -> Result<(), AppError> {
        let mut task = self.get_task(&task_id).await?;
        task.status = TaskStatus::Completed;
        task.updated_at = Utc::now();

        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let task_json = serde_json::to_string(&task)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        redis_conn.set_ex(format!("task:{}", task_id), task_json, 3600).await
            .map_err(|e| AppError::CacheError(format!("Failed to update task in Redis: {}", e)))?;

        Ok(())
    }

    async fn mark_task_failed(&self, task_id: String, error: String) -> Result<(), AppError> {
        let mut task = self.get_task(&task_id).await?;
        task.status = TaskStatus::Failed;
        task.error = Some(error);
        task.updated_at = Utc::now();

        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let task_json = serde_json::to_string(&task)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        redis_conn.set_ex(format!("task:{}", task_id), task_json, 3600).await
            .map_err(|e| AppError::CacheError(format!("Failed to update task in Redis: {}", e)))?;

        Ok(())
    }

    async fn send_results(&self, task_id: &str, products: Vec<Product>) -> Result<(), AppError> {
        // Store results in Redis
        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let results_json = serde_json::to_string(&products)
            .map_err(|e| AppError::SerializationError(e.to_string()))?;

        redis_conn.set_ex(format!("results:{}", task_id), results_json, 3600).await
            .map_err(|e| AppError::CacheError(format!("Failed to store results in Redis: {}", e)))?;

        // Send results through response channel if it exists
        if let Some(tx) = self.response_channels.lock().await.remove(task_id) {
            let _ = tx.send(products);
        }

        Ok(())
    }

    async fn get_results(&self, task_id: &str) -> Result<Option<Vec<Product>>, AppError> {
        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;

        let results: Option<String> = redis_conn.get(format!("results:{}", task_id)).await
            .map_err(|e| AppError::CacheError(format!("Failed to get results from Redis: {}", e)))?;

        if let Some(results_json) = results {
            let products: Vec<Product> = serde_json::from_str(&results_json)
                .map_err(|e| AppError::SerializationError(format!("Failed to deserialize results: {}", e)))?;
            Ok(Some(products))
        } else {
            Ok(None)
        }
    }
} 