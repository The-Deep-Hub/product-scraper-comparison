use crate::domain::ports::outbound::{CachePort, QueuePort, ScraperPort};
use crate::domain::models::Store;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use lapin::{
    options::*, types::FieldTable,
    Connection, ConnectionProperties,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, error};

const PREFETCH_COUNT: u16 = 3;
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const DEFAULT_LIMIT: Option<usize> = Some(100); // Default limit for product scraping

pub struct TaskProcessor {
    amqp_url: String,
    cache_port: Arc<dyn CachePort>,
    queue_port: Arc<dyn QueuePort>,
    scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
}

impl TaskProcessor {
    pub fn new(
        amqp_url: String,
        cache_port: Arc<dyn CachePort>,
        queue_port: Arc<dyn QueuePort>,
        scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
    ) -> Self {
        Self {
            amqp_url,
            cache_port,
            queue_port,
            scrapers,
        }
    }

    fn get_queue_name(store: &Store) -> String {
        format!("scraper_tasks_{}", store.to_string().to_lowercase())
    }

    pub async fn run(&self) -> std::io::Result<()> {
        loop {
            match self.connect_and_process().await {
                Ok(_) => {
                    info!("Connection closed, reconnecting...");
                    sleep(RECONNECT_DELAY).await;
                }
                Err(e) => {
                    error!("Error processing tasks: {}", e);
                    sleep(RECONNECT_DELAY).await;
                }
            }
        }
    }

    async fn connect_and_process(&self) -> std::io::Result<()> {
        let connection = Connection::connect(
            &self.amqp_url,
            ConnectionProperties::default(),
        ).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let channel = connection.create_channel().await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Set QoS
        channel.basic_qos(PREFETCH_COUNT, BasicQosOptions::default())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut consumer_futures = FuturesUnordered::new();
        
        // Create a consumer for each store's queue
        for scraper in self.scrapers.iter() {
            let store = scraper.get_store();
            let queue_name = Self::get_queue_name(&store);
            
            // Declare queue for this store
            info!("Declaring queue for store {}: {}", store, queue_name);
            channel.queue_declare(
                &queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..QueueDeclareOptions::default()
                },
                FieldTable::default(),
            ).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            info!("Starting consumer for store {} queue: {}", store, queue_name);
            let mut consumer = channel.basic_consume(
                &queue_name,
                &format!("task_processor_{}", queue_name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            ).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let cache_port = self.cache_port.clone();
            let scraper = scraper.clone();
            
            // Process messages for this store's queue
            let future = async move {
                info!("Consumer started for store {} queue: {}", store, queue_name);
                
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            let query = match String::from_utf8(delivery.data.clone()) {
                                Ok(s) => s,
                                Err(e) => {
                                    error!("Failed to parse query data: {}", e);
                                    let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                    continue;
                                }
                            };
                            
                            let start_time = Instant::now();
                            info!("Starting task for store {} and query: {} at {:?}", store, query, start_time);
                            
                            match scraper.scrape_products(&query, DEFAULT_LIMIT).await {
                                Ok(products) => {
                                    let scraping_duration = start_time.elapsed();
                                    info!(
                                        "Found {} products for store {} and query: {} in {:?}",
                                        products.len(), store, query, scraping_duration
                                    );
                                    
                                    let cache_start = Instant::now();
                                    // Cache both store-specific and all products
                                    if let Err(e) = cache_port.cache_store_products(&query, &store, &products).await {
                                        error!("Failed to cache store products: {}", e);
                                        let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                        continue;
                                    }
                                    
                                    // Also cache in the all-products cache
                                    if let Err(e) = cache_port.cache_products(&query, &products).await {
                                        error!("Failed to cache all products: {}", e);
                                        // Don't reject if at least store-specific cache worked
                                    }
                                    
                                    let cache_duration = cache_start.elapsed();
                                    info!(
                                        "Stored results for store: {}, query: {} - Scraping took {:?}, Caching took {:?}",
                                        store, query, scraping_duration, cache_duration
                                    );
                                    let _ = delivery.ack(BasicAckOptions::default()).await;
                                }
                                Err(e) => {
                                    error!("Failed to scrape products for store {} after {:?}: {}", 
                                        store, start_time.elapsed(), e);
                                    let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error receiving message: {}", e);
                        }
                    }
                }
                
                Ok::<_, std::io::Error>(())
            };
            
            consumer_futures.push(future);
        }
        
        // Wait for all consumers to complete
        while let Some(result) = consumer_futures.next().await {
            if let Err(e) = result {
                error!("Consumer error: {}", e);
            }
        }
        
        Ok(())
    }
} 