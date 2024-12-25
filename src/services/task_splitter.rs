use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, error};
use lapin::{
    options::*, types::FieldTable, BasicProperties,
    Channel,
};

use crate::{
    error::{AppResult, AppError},
    models::{
        task::{MainTask, StoreTask},
        store::Store,
    },
    services::cache::CacheService,
};

const STORE_QUEUE_PREFIX: &str = "store_tasks";

#[async_trait]
pub trait TaskSplitterService: Send + Sync {
    async fn split_task(&self, main_task: &MainTask) -> AppResult<()>;
    async fn setup_queues(&self) -> AppResult<()>;
}

pub struct RabbitMQTaskSplitter {
    channel: Channel,
    cache_service: Arc<dyn CacheService>,
}

impl RabbitMQTaskSplitter {
    pub fn new(channel: Channel, cache_service: Arc<dyn CacheService>) -> Self {
        Self {
            channel,
            cache_service,
        }
    }

    fn get_store_queue_name(store: &Store) -> String {
        format!("{}_{}", STORE_QUEUE_PREFIX, store.to_string().to_lowercase())
    }
}

#[async_trait]
impl TaskSplitterService for RabbitMQTaskSplitter {
    async fn setup_queues(&self) -> AppResult<()> {
        info!("Setting up store-specific queues...");
        
        // Setup queues for each store
        for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            let queue_name = Self::get_store_queue_name(store);
            
            // Declare the queue
            self.channel.queue_declare(
                &queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            ).await.map_err(|e| {
                error!("Failed to declare queue {}: {}", queue_name, e);
                AppError::QueueError(format!("Failed to declare queue: {}", e))
            })?;
            
            info!("Declared queue: {}", queue_name);
        }
        
        Ok(())
    }

    async fn split_task(&self, main_task: &MainTask) -> AppResult<()> {
        info!("Splitting task {} for stores: {:?}", main_task.id, main_task.pending_stores);
        
        // Create and publish store-specific tasks
        for store in &main_task.pending_stores {
            let store_task = StoreTask::new(
                main_task.id.clone(),
                main_task.query.clone(),
                store.clone(),
            );
            
            // Store the task in cache
            self.cache_service.set_store_task(&store_task).await?;
            
            // Publish to store-specific queue
            let queue_name = Self::get_store_queue_name(store);
            let payload = serde_json::to_vec(&store_task)?;
            
            self.channel.basic_publish(
                "",
                &queue_name,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default()
                    .with_delivery_mode(2), // persistent delivery
            ).await.map_err(|e| {
                error!("Failed to publish task to queue {}: {}", queue_name, e);
                AppError::QueueError(format!("Failed to publish task: {}", e))
            })?;
            
            info!("Published task {} to queue {}", store_task.id, queue_name);
        }
        
        Ok(())
    }
} 