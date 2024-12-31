use crate::domain::ports::outbound::{CachePort, QueuePort};
use crate::domain::services::scraper::ScraperService;
use crate::workers::tasks::{StoreTask, StoreResult};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use lapin::{
    options::*, types::FieldTable,
    Connection, ConnectionProperties,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

const PREFETCH_COUNT: u16 = 3;
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const DEFAULT_LIMIT: Option<usize> = Some(100); // Default limit for product scraping

pub struct TaskProcessor {
    amqp_url: String,
    store_queues: Vec<String>,
    cache_port: Arc<dyn CachePort>,
    queue_port: Arc<dyn QueuePort>,
    scraper_service: Arc<ScraperService>,
}

impl TaskProcessor {
    pub fn new(
        amqp_url: String,
        store_queues: Vec<String>,
        cache_port: Arc<dyn CachePort>,
        queue_port: Arc<dyn QueuePort>,
        scraper_service: Arc<ScraperService>,
    ) -> Self {
        Self {
            amqp_url,
            store_queues,
            cache_port,
            queue_port,
            scraper_service,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match self.setup_and_run_consumers().await {
                Ok(_) => {
                    error!("Consumers stopped unexpectedly");
                }
                Err(e) => {
                    error!("Consumer error: {}. Reconnecting in {} seconds...", e, RECONNECT_DELAY.as_secs());
                }
            }
            
            sleep(RECONNECT_DELAY).await;
            info!("Attempting to reconnect...");
        }
    }

    async fn setup_and_run_consumers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = Connection::connect(
            &self.amqp_url,
            ConnectionProperties::default(),
        ).await?;
        
        let channel = conn.create_channel().await?;
        channel.basic_qos(PREFETCH_COUNT, BasicQosOptions::default()).await?;

        let mut consumer_futures = FuturesUnordered::new();
        
        // Start a consumer for each store queue
        for queue_name in &self.store_queues {
            info!("Starting consumer for queue: {}", queue_name);
            let mut consumer = channel.basic_consume(
                queue_name,
                &format!("task_processor_{}", queue_name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            ).await?;

            let channel_clone = channel.clone();
            let queue_name = queue_name.clone();
            let cache_port = self.cache_port.clone();
            let scraper_service = self.scraper_service.clone();
            
            // Process messages from this queue
            let future = async move {
                info!("Consumer started for queue: {}", queue_name);
                
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            let task: StoreTask = match serde_json::from_slice(&delivery.data) {
                                Ok(task) => task,
                                Err(e) => {
                                    error!("Failed to parse task from queue {}: {}", queue_name, e);
                                    let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                    continue;
                                }
                            };
                            
                            info!("Received task: {} for query: {} from queue: {}", 
                                task.id, task.query, queue_name);
                            
                            let result = match scraper_service.scrape_products(&task.query, DEFAULT_LIMIT).await {
                                Ok(products) => {
                                    info!("Found {} products for query: {}", products.len(), task.query);
                                    
                                    // Cache the products
                                    if let Err(e) = cache_port.cache_products(&task.query, &products).await {
                                        error!("Failed to cache products for task {}: {}", task.id, e);
                                        let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                        continue;
                                    }
                                    
                                    info!("Stored results for task {}", task.id);
                                    let _ = delivery.ack(BasicAckOptions::default()).await;
                                }
                                Err(e) => {
                                    error!("Failed to scrape products for task {}: {}", task.id, e);
                                    let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                }
                            };
                        }
                        Err(e) => {
                            error!("Error receiving message from queue {}: {}", queue_name, e);
                        }
                    }
                }
                
                Ok::<_, Box<dyn std::error::Error>>(())
            };
            
            consumer_futures.push(future);
        }

        // Wait for all consumers to complete (they should never complete unless there's an error)
        while let Some(result) = consumer_futures.next().await {
            result?;
        }
        
        Ok(())
    }
}
