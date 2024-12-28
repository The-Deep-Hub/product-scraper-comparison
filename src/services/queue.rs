use async_trait::async_trait;
use lapin::{
    options::*, types::{FieldTable, AMQPValue}, BasicProperties,
    Connection, Channel,
};
use serde::{Serialize, Deserialize};
use tracing::info;
use std::any::Any;
use futures_lite::StreamExt;
use uuid::Uuid;

use crate::{
    error::{AppResult, AppError},
    models::store::Store,
};

const STORE_QUEUE_PREFIX: &str = "store_tasks";
const DL_EXCHANGE: &str = "dl.store_tasks";

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub query: String,
}

#[async_trait]
pub trait QueueService: Send + Sync {
    async fn setup(&self) -> AppResult<()>;
    async fn publish(&self, queue: &str, message: Box<dyn Any + Send + Sync>) -> AppResult<()>;
    async fn consume(&self, queue: &str) -> AppResult<Option<Box<dyn Any + Send + Sync>>>;
    async fn mark_task_completed(&self, task_id: String) -> AppResult<()>;
    async fn mark_task_failed(&self, task_id: String, error: String) -> AppResult<()>;
    async fn get_pending_task(&self) -> AppResult<Option<Task>>;
    async fn create_task(&self, query: String) -> AppResult<String>;
}

pub struct RabbitMQQueue {
    channel: Channel,
}

impl RabbitMQQueue {
    pub async fn new() -> AppResult<Self> {
        let rabbitmq_url = format!(
            "amqp://{}:{}@{}:{}",
            std::env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string()),
            std::env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string()),
            std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string()),
            std::env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string()),
        );

        info!("Connecting to RabbitMQ at: {}", rabbitmq_url);
        let conn = Connection::connect(
            &rabbitmq_url,
            Default::default(),
        ).await.map_err(|e| AppError::QueueError(format!("Failed to connect to RabbitMQ: {}", e)))?;

        let channel = conn.create_channel().await
            .map_err(|e| AppError::QueueError(format!("Failed to create channel: {}", e)))?;

        Ok(Self { channel })
    }

    pub fn get_store_queue_name(store: &Store) -> String {
        format!("{}_{}", STORE_QUEUE_PREFIX, store.to_string().to_lowercase())
    }
}

#[async_trait]
impl QueueService for RabbitMQQueue {
    async fn setup(&self) -> AppResult<()> {
        info!("Setting up store-specific queues...");
        
        // Setup queues for each store
        for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            let queue_name = Self::get_store_queue_name(store);
            info!("Setting up {} queue...", queue_name);
            
            let mut args = FieldTable::default();
            args.insert(
                "x-dead-letter-exchange".into(),
                AMQPValue::ShortString(DL_EXCHANGE.into())
            );
            args.insert("x-message-ttl".into(), 300000_i32.into()); // 5 minutes TTL
            
            self.channel.queue_declare(
                &queue_name,
                QueueDeclareOptions::default(),
                args,
            ).await.map_err(|e| AppError::QueueError(format!("Failed to declare queue {}: {}", queue_name, e)))?;
        }
        
        info!("RabbitMQ setup completed successfully");
        Ok(())
    }

    async fn publish(&self, queue: &str, message: Box<dyn Any + Send + Sync>) -> AppResult<()> {
        let message = message.downcast_ref::<Vec<u8>>()
            .ok_or_else(|| AppError::QueueError("Failed to downcast message to bytes".into()))?;

        self.channel.basic_publish(
            "",
            queue,
            BasicPublishOptions::default(),
            message,
            BasicProperties::default(),
        ).await.map_err(|e| AppError::QueueError(format!("Failed to publish message: {}", e)))?;

        Ok(())
    }

    async fn consume(&self, queue: &str) -> AppResult<Option<Box<dyn Any + Send + Sync>>> {
        let mut consumer = self.channel.basic_consume(
            queue,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        ).await.map_err(|e| AppError::QueueError(format!("Failed to create consumer: {}", e)))?;

        if let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let message: Box<dyn Any + Send + Sync> = Box::new(delivery.data);

                    self.channel.basic_ack(
                        delivery.delivery_tag,
                        BasicAckOptions::default(),
                    ).await.map_err(|e| AppError::QueueError(format!("Failed to acknowledge message: {}", e)))?;

                    Ok(Some(message))
                }
                Err(e) => Err(AppError::QueueError(format!("Failed to get delivery: {}", e))),
            }
        } else {
            Ok(None)
        }
    }

    async fn mark_task_completed(&self, _task_id: String) -> AppResult<()> {
        // For now, we don't need to do anything since we already ack the message
        Ok(())
    }

    async fn mark_task_failed(&self, _task_id: String, _error: String) -> AppResult<()> {
        // For now, we don't need to do anything since we already ack the message
        Ok(())
    }

    async fn get_pending_task(&self) -> AppResult<Option<Task>> {
        if let Some(message) = self.consume(STORE_QUEUE_PREFIX).await? {
            if let Ok(query) = message.downcast::<String>() {
                return Ok(Some(Task {
                    id: Uuid::new_v4().to_string(),
                    query: *query,
                }));
            }
        }
        Ok(None)
    }

    async fn create_task(&self, query: String) -> AppResult<String> {
        let task_id = Uuid::new_v4().to_string();
        
        // Create a task for each store
        for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            let queue_name = Self::get_store_queue_name(store);
            let message = Box::new(query.clone());
            self.publish(&queue_name, message).await?;
        }
        
        Ok(task_id)
    }
}