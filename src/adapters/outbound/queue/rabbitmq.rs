use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;
use lapin::{
    options::*, types::{FieldTable, AMQPValue}, BasicProperties,
    Connection, ConnectionProperties, Channel,
};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tracing::{info, error};
use uuid::Uuid;

use crate::{
    domain::{
        models::{DomainError, Store, DomainResult},
        ports::outbound::QueuePort,
    },
    adapters::inbound::worker::processor::ScrapeTask,
};

const MESSAGE_TTL: i32 = 300000; // 5 minutes in milliseconds
const PREFETCH_COUNT: u16 = 1; // Number of messages to prefetch per consumer

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub query: String,
    pub store: Option<Store>,
    pub num_products: Option<usize>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
pub struct RabbitMQAdapter {
    // We need to keep the connection alive as long as we have active channels.
    // While we don't directly use it, dropping it would invalidate all channels.
    #[allow(dead_code)]
    connection: Arc<Connection>,
    channel: Arc<Channel>,
    base_queue_name: String,
}

impl RabbitMQAdapter {
    pub async fn new(amqp_url: String, base_queue_name: String) -> Result<Self, DomainError> {
        info!("Connecting to RabbitMQ at: {}", amqp_url);
        
        let connection = Connection::connect(
            &amqp_url,
            ConnectionProperties::default(),
        ).await.map_err(|e| DomainError::Queue(format!("Failed to connect to RabbitMQ: {}", e)))?;

        let channel = connection.create_channel().await
            .map_err(|e| DomainError::Queue(format!("Failed to create channel: {}", e)))?;

        // Set QoS for the channel
        channel.basic_qos(PREFETCH_COUNT, BasicQosOptions::default())
            .await
            .map_err(|e| DomainError::Queue(format!("Failed to set QoS: {}", e)))?;

        let adapter = Self {
            connection: Arc::new(connection),
            channel: Arc::new(channel),
            base_queue_name,
        };

        // Declare queues for all stores
        for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            adapter.declare_store_queue(store).await?;
        }

        Ok(adapter)
    }

    fn get_store_queue_name(&self, store: &Store) -> String {
        format!("{}_{}", self.base_queue_name, store.to_string().to_lowercase())
    }

    async fn declare_store_queue(&self, store: &Store) -> DomainResult<()> {
        let queue_name = self.get_store_queue_name(store);
        
        // Declare the queue with message TTL
        let mut args = FieldTable::default();
        args.insert("x-message-ttl".into(), AMQPValue::LongInt(MESSAGE_TTL));

        self.channel.queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
            args,
        ).await.map_err(|e| DomainError::Queue(format!("Failed to declare queue {}: {}", queue_name, e)))?;

        info!("Declared queue: {}", queue_name);
        Ok(())
    }

    async fn publish_task(&self, task: Task) -> DomainResult<()> {
        let payload = serde_json::to_vec(&task)
            .map_err(|e| DomainError::Queue(format!("Failed to serialize task: {}", e)))?;

        let queue_name = match &task.store {
            Some(store) => self.get_store_queue_name(store),
            None => {
                // For broadcast messages, publish to all store queues
                for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
                    let store_queue = self.get_store_queue_name(store);
                    self.channel.basic_publish(
                        "",
                        &store_queue,
                        BasicPublishOptions::default(),
                        &payload,
                        BasicProperties::default(),
                    ).await.map_err(|e| DomainError::Queue(format!("Failed to publish message: {}", e)))?;
                }
                return Ok(());
            }
        };

        self.channel.basic_publish(
            "",
            &queue_name,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        ).await.map_err(|e| DomainError::Queue(format!("Failed to publish message: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl QueuePort for RabbitMQAdapter {
    async fn enqueue_scrape_job(&self, task_id: &str, query: &str, store: Store, num_products: Option<usize>) -> DomainResult<()> {
        let task = Task {
            id: task_id.to_string(),
            query: query.to_string(),
            store: Some(store),
            num_products,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.publish_task(task).await
    }

    async fn process_scrape_job(&self, query: &str, num_products: Option<usize>) -> DomainResult<()> {
        // For all-stores case, we just enqueue the job for each store
        for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            let task_id = Uuid::new_v4().to_string();
            self.enqueue_scrape_job(&task_id, query, store.clone(), num_products).await?;
        }
        Ok(())
    }

    async fn process_store_scrape_job(&self, query: &str, store: &Store, _num_products: Option<usize>) -> DomainResult<Option<String>> {
        let queue_name = self.get_store_queue_name(store);
        let mut consumer = self.channel
            .basic_consume(
                &queue_name,
                &format!("consumer-{}", Uuid::new_v4()),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| DomainError::Queue(format!("Failed to create consumer: {}", e)))?;

        while let Some(delivery_result) = consumer.next().await {
            match delivery_result {
                Ok(delivery) => {
                    let payload = String::from_utf8_lossy(&delivery.data);
                    let task: Task = serde_json::from_str(&payload)
                        .map_err(|e| DomainError::Queue(format!("Failed to parse task: {}", e)))?;

                    if task.query == query {
                        delivery.ack(BasicAckOptions::default())
                            .await
                            .map_err(|e| DomainError::Queue(format!("Failed to acknowledge message: {}", e)))?;

                        return Ok(Some(task.query));
                    }
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    return Err(DomainError::Queue(format!("Failed to receive message: {}", e)));
                }
            }
        }

        Ok(None)
    }

    async fn get_value(&self, _key: &str) -> DomainResult<Option<String>> {
        // This adapter doesn't support key-value storage
        Ok(None)
    }

    async fn consume_messages(&self, store: &Store, tx: mpsc::Sender<ScrapeTask>) -> DomainResult<()> {
        let queue_name = self.get_store_queue_name(store);
        let mut consumer = self.channel
            .basic_consume(
                &queue_name,
                &format!("consumer-{}", Uuid::new_v4()),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| DomainError::Queue(format!("Failed to create consumer: {}", e)))?;

        while let Some(delivery_result) = consumer.next().await {
            match delivery_result {
                Ok(delivery) => {
                    let payload = String::from_utf8_lossy(&delivery.data);
                    let task: Task = serde_json::from_str(&payload)
                        .map_err(|e| DomainError::Queue(format!("Failed to parse task: {}", e)))?;

                    let scrape_task = ScrapeTask {
                        query: task.query,
                        store: store.clone(),
                        num_products: task.num_products,
                    };

                    if let Err(e) = tx.send(scrape_task).await {
                        error!("Failed to send task to processor: {}", e);
                        return Err(DomainError::Queue(format!("Failed to send task to processor: {}", e)));
                    }

                    delivery.ack(BasicAckOptions::default())
                        .await
                        .map_err(|e| DomainError::Queue(format!("Failed to acknowledge message: {}", e)))?;
                }
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    return Err(DomainError::Queue(format!("Failed to receive message: {}", e)));
                }
            }
        }

        Ok(())
    }
} 