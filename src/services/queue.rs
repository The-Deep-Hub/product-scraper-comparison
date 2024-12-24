use async_trait::async_trait;
use futures_util::StreamExt;
use lapin::{
    options::{BasicPublishOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

use crate::error::AppResult;

const QUEUE_NAME: &str = "product_details";

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub url: String,
    pub status: TaskStatus,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[async_trait::async_trait]
pub trait QueueService: Send + Sync {
    async fn create_task(&self, query: &str) -> AppResult<String>;
    async fn get_pending_tasks(&self, limit: usize) -> AppResult<Vec<Task>>;
    async fn mark_task_completed(&self, task_id: &str) -> AppResult<()>;
    async fn mark_task_failed(&self, task_id: &str) -> AppResult<()>;
}

#[derive(Clone)]
pub struct RabbitMQQueue {
    channel: Channel,
}

impl RabbitMQQueue {
    pub async fn new() -> AppResult<Self> {
        let rabbitmq_uri = format!(
            "amqp://{}:{}@{}:{}{}",
            std::env::var("RABBITMQ_USER").expect("RABBITMQ_USER must be set"),
            std::env::var("RABBITMQ_PASSWORD").expect("RABBITMQ_PASSWORD must be set"),
            std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string()),
            std::env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string()),
            std::env::var("RABBITMQ_VHOST").unwrap_or_else(|_| "/".to_string())
        );

        let conn = Connection::connect(
            &rabbitmq_uri,
            ConnectionProperties::default(),
        ).await?;

        let channel = conn.create_channel().await?;

        // Declare the queue
        channel
            .queue_declare(
                QUEUE_NAME,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(Self { channel })
    }
}

#[async_trait]
impl QueueService for RabbitMQQueue {
    async fn create_task(&self, query: &str) -> AppResult<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            url: query.to_string(),
            status: TaskStatus::Pending,
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
        };

        let payload = serde_json::to_vec(&task)?;
        self.channel
            .basic_publish(
                "",
                QUEUE_NAME,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await?;

        Ok(task_id)
    }

    async fn get_pending_tasks(&self, limit: usize) -> AppResult<Vec<Task>> {
        let mut tasks = Vec::new();
        let mut consumer = self.channel
            .basic_consume(
                QUEUE_NAME,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            if tasks.len() >= limit {
                break;
            }

            let delivery = delivery?;
            let task: Task = serde_json::from_slice(&delivery.data)?;
            tasks.push(task);
            delivery.ack(Default::default()).await?;
        }

        Ok(tasks)
    }

    async fn mark_task_completed(&self, _task_id: &str) -> AppResult<()> {
        // In a real implementation, you would update the task status in a database
        Ok(())
    }

    async fn mark_task_failed(&self, _task_id: &str) -> AppResult<()> {
        // In a real implementation, you would update the task status in a database
        Ok(())
    }
} 