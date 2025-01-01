use crate::domain::ports::outbound::{CachePort, QueuePort, ScraperPort};
use crate::domain::models::Store;
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
    queue_names: Vec<String>,
    cache_port: Arc<dyn CachePort>,
    queue_port: Arc<dyn QueuePort>,
    scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
}

impl TaskProcessor {
    pub fn new(
        amqp_url: String,
        queue_names: Vec<String>,
        cache_port: Arc<dyn CachePort>,
        queue_port: Arc<dyn QueuePort>,
        scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
    ) -> Self {
        Self {
            amqp_url,
            queue_names,
            cache_port,
            queue_port,
            scrapers,
        }
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
        
        // Start a consumer for each queue
        for queue_name in &self.queue_names {
            info!("Starting consumer for queue: {}", queue_name);
            let mut consumer = channel.basic_consume(
                queue_name,
                &format!("task_processor_{}", queue_name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            ).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let cache_port = self.cache_port.clone();
            let scrapers = self.scrapers.clone();
            
            // Process messages from this queue
            let future = async move {
                info!("Consumer started for queue: {}", queue_name);
                
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            let task_str = match String::from_utf8(delivery.data.clone()) {
                                Ok(s) => s,
                                Err(e) => {
                                    error!("Failed to parse task data: {}", e);
                                    let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                    continue;
                                }
                            };
                            
                            // Parse store and query from task
                            let parts: Vec<&str> = task_str.split(':').collect();
                            if parts.len() != 2 {
                                error!("Invalid task format: {}", task_str);
                                let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                continue;
                            }
                            
                            let store = parts[0];
                            let query = parts[1];
                            
                            info!("Processing task for store: {} and query: {}", store, query);
                            
                            // Handle "all" store case
                            if store == "all" {
                                let mut all_products = Vec::new();
                                
                                // Scrape products from all scrapers
                                for scraper in scrapers.iter() {
                                    match scraper.scrape_products(query, DEFAULT_LIMIT).await {
                                        Ok(products) => {
                                            info!("Found {} products for store {} and query: {}", 
                                                products.len(), 
                                                scraper.get_store(), 
                                                query
                                            );
                                            all_products.extend(products);
                                        }
                                        Err(e) => {
                                            error!("Failed to scrape products from {}: {}", scraper.get_store(), e);
                                            // Continue with other scrapers even if one fails
                                            continue;
                                        }
                                    }
                                }
                                
                                // Cache all products
                                if let Err(e) = cache_port.cache_products(query, &all_products).await {
                                    error!("Failed to cache products: {}", e);
                                    let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                    continue;
                                }
                                
                                info!("Stored results for all stores, query: {}", query);
                                let _ = delivery.ack(BasicAckOptions::default()).await;
                                continue;
                            }
                            
                            // Handle store-specific case
                            let store_enum = match Store::from_str(store) {
                                Some(s) => s,
                                None => {
                                    error!("Invalid store name: {}", store);
                                    let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                    continue;
                                }
                            };
                            
                            let scraper = scrapers.iter().find(|s| s.get_store() == store_enum);
                            
                            match scraper {
                                Some(scraper) => {
                                    match scraper.scrape_products(query, DEFAULT_LIMIT).await {
                                        Ok(products) => {
                                            info!("Found {} products for query: {}", products.len(), query);
                                            
                                            // Cache both store-specific and all products
                                            if let Err(e) = cache_port.cache_store_products(query, &store_enum, &products).await {
                                                error!("Failed to cache store products: {}", e);
                                                let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                                continue;
                                            }
                                            
                                            // Also cache in the all-products cache
                                            if let Err(e) = cache_port.cache_products(query, &products).await {
                                                error!("Failed to cache all products: {}", e);
                                                // Don't reject if at least store-specific cache worked
                                            }
                                            
                                            info!("Stored results for store: {}, query: {}", store, query);
                                            let _ = delivery.ack(BasicAckOptions::default()).await;
                                        }
                                        Err(e) => {
                                            error!("Failed to scrape products: {}", e);
                                            let _ = delivery.reject(BasicRejectOptions { requeue: false }).await;
                                        }
                                    }
                                }
                                None => {
                                    error!("No scraper found for store: {}", store);
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
