use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::domain::models::DomainError;
use crate::domain::ports::{
    CachePort,
    ScraperPort,
    QueuePort,
};

pub struct ScraperWorkerService {
    cache: Arc<dyn CachePort>,
    scrapers: Vec<Arc<dyn ScraperPort>>,
    queue: Arc<dyn QueuePort>,
    cache_ttl: Duration,
}

impl ScraperWorkerService {
    pub fn new(
        cache: Arc<dyn CachePort>,
        scrapers: Vec<Arc<dyn ScraperPort>>,
        queue: Arc<dyn QueuePort>,
        cache_ttl: Duration,
    ) -> Self {
        Self {
            cache,
            scrapers,
            queue,
            cache_ttl,
        }
    }

    async fn get_scraper_for_store(&self, store_name: &str) -> Option<Arc<dyn ScraperPort>> {
        self.scrapers
            .iter()
            .find(|scraper| scraper.get_store().as_str() == store_name)
            .cloned()
    }

    pub async fn process_jobs(&self) -> Result<(), DomainError> {
        loop {
            match self.queue.process_next_job().await {
                Ok(Some(job)) => {
                    info!("Processing scrape job for query: {}", job.query);
                    
                    let scrapers = if let Some(store) = job.store {
                        if let Some(scraper) = self.get_scraper_for_store(store.as_str()).await {
                            vec![scraper]
                        } else {
                            warn!("No scraper found for store: {}", store);
                            continue;
                        }
                    } else {
                        self.scrapers.clone()
                    };

                    let mut all_products = Vec::new();
                    for scraper in scrapers {
                        match scraper.scrape_products(&job.query, None).await {
                            Ok(products) => {
                                info!(
                                    "Found {} products from {} for query: {}",
                                    products.len(),
                                    scraper.get_store(),
                                    job.query
                                );
                                all_products.extend(products);
                            }
                            Err(e) => {
                                error!(
                                    "Failed to scrape products from {}: {}",
                                    scraper.get_store(),
                                    e
                                );
                            }
                        }
                    }

                    if !all_products.is_empty() {
                        if let Err(e) = self.cache.cache_products(&job.query, all_products, self.cache_ttl).await {
                            error!("Failed to cache products: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    // No jobs available, wait a bit before checking again
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Error processing job: {}", e);
                    // Wait a bit before retrying
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
} 