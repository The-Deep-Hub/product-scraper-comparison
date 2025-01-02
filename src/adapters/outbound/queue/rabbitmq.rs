use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;
use futures_util::TryStreamExt;
use lapin::{
    options::*, types::{FieldTable, AMQPValue}, BasicProperties,
    Connection, ConnectionProperties, Channel, Consumer,
};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tracing::{info, error};
use uuid::Uuid;

use crate::domain::{
    models::{DomainError, Store, DomainResult},
    ports::outbound::QueuePort,
};

const MESSAGE_TTL: i32 = 300000; // 5 minutes in milliseconds
const PREFETCH_COUNT: u16 = 1; // Number of messages to prefetch per consumer

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
    async fn enqueue_scrape_job(&self, query: &str) -> DomainResult<()> {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            query: query.to_string(),
            store: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.publish_task(task).await
    }

    async fn enqueue_store_scrape_job(&self, query: &str, store: &Store) -> DomainResult<()> {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            query: query.to_string(),
            store: Some(store.clone()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.publish_task(task).await
    }

    async fn process_scrape_job(&self, query: &str) -> DomainResult<()> {
        // For all-stores case, we just enqueue the job
        self.enqueue_scrape_job(query).await
    }

    async fn process_store_scrape_job(&self, query: &str, store: &Store) -> DomainResult<Option<String>> {
        // Enqueue the job for a specific store
        self.enqueue_store_scrape_job(query, store).await?;
        Ok(Some(query.to_string()))
    }

    async fn get_value(&self, _key: &str) -> DomainResult<Option<String>> {
        // This adapter doesn't support key-value storage
        Ok(None)
    }

    async fn consume_messages(&self, store: &Store, tx: mpsc::Sender<(String, Store)>) -> DomainResult<()> {
        let queue_name = self.get_store_queue_name(store);
        info!("Starting consumer for store {} on queue {}", store, queue_name);
        
        let consumer = self.channel.basic_consume(
            &queue_name,
            &format!("scraper_consumer_{}", store),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        ).await.map_err(|e| DomainError::Queue(format!("Failed to create consumer: {}", e)))?;

        let mut consumer_stream = consumer.into_stream();

        while let Some(delivery) = consumer_stream.next().await {
            match delivery {
                Ok(delivery) => {
                    let task: Task = match serde_json::from_slice(&delivery.data) {
                        Ok(task) => task,
                        Err(e) => {
                            error!("Failed to deserialize task: {}", e);
                            delivery.ack(BasicAckOptions::default()).await
                                .map_err(|e| DomainError::Queue(format!("Failed to ack message: {}", e)))?;
                            continue;
                        }
                    };

                    if let Err(e) = tx.send((task.query, store.clone())).await {
                        error!("Failed to send task to processor: {}", e);
                    }

                    delivery.ack(BasicAckOptions::default()).await
                        .map_err(|e| DomainError::Queue(format!("Failed to ack message: {}", e)))?;
                }
                Err(e) => {
                    error!("Failed to receive message: {}", e);
                }
            }
        }

        Ok(())
    }
} 