use crate::domain::ports::outbound::{CachePort, QueuePort, ScraperPort};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::{info, error};

const DEFAULT_LIMIT: Option<usize> = Some(100); // Default limit for product scraping
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const CHANNEL_BUFFER_SIZE: usize = 100;

pub struct TaskProcessor {
    cache_port: Arc<dyn CachePort>,
    queue_port: Arc<dyn QueuePort>,
    scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
}

impl TaskProcessor {
    pub fn new(
        cache_port: Arc<dyn CachePort>,
        queue_port: Arc<dyn QueuePort>,
        scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
    ) -> Self {
        Self {
            cache_port,
            queue_port,
            scrapers,
        }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let (tx, mut rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        let mut tasks = JoinSet::new();

        // Start consumers for each store
        for scraper in self.scrapers.iter() {
            let store = scraper.get_store();
            let queue_port = self.queue_port.clone();
            let tx = tx.clone();

            tasks.spawn(async move {
                loop {
                    if let Err(e) = queue_port.consume_messages(&store, tx.clone()).await {
                        error!("Consumer error for store {}: {}", store, e);
                        sleep(RECONNECT_DELAY).await;
                    }
                }
            });
        }

        // Process messages from the channel
        while let Some((query, store)) = rx.recv().await {
            let scraper = self.scrapers.iter()
                .find(|s| s.get_store() == store)
                .cloned();

            if let Some(scraper) = scraper {
                let cache_port = self.cache_port.clone();
                let start_time = Instant::now();

                tasks.spawn(async move {
                    info!("Processing task for store {} and query: {}", store, query);
                    
                    // Scrape products
                    match scraper.scrape_products(&query, DEFAULT_LIMIT).await {
                        Ok(products) => {
                            let scraping_duration = start_time.elapsed();
                            info!(
                                "Found {} products for store {} and query: {} in {:?}",
                                products.len(), store, query, scraping_duration
                            );
                            
                            // Cache the results
                            let cache_start = Instant::now();
                            if let Err(e) = cache_port.cache_store_products(&query, &store, &products).await {
                                error!("Failed to cache store products: {}", e);
                                return;
                            }
                            
                            // Also cache in the all-products cache
                            if let Err(e) = cache_port.cache_products(&query, &products).await {
                                error!("Failed to cache all products: {}", e);
                                // Don't fail if at least store-specific cache worked
                            }
                            
                            let cache_duration = cache_start.elapsed();
                            info!(
                                "Stored results for store: {}, query: {} - Scraping took {:?}, Caching took {:?}",
                                store, query, scraping_duration, cache_duration
                            );
                        }
                        Err(e) => {
                            error!("Failed to scrape products for store {} after {:?}: {}", 
                                store, start_time.elapsed(), e);
                        }
                    }
                });
            }
        }

        Ok(())
    }
} 