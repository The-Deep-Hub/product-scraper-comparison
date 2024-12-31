use std::sync::Arc;
use tracing::{info, warn};

use crate::domain::models::{Product, Store, DomainResult, DomainError};
use crate::domain::ports::outbound::{CachePort, QueuePort, ScraperPort};
use crate::domain::events::DomainEvent;

pub struct SearchService {
    cache: Arc<dyn CachePort>,
    queue: Arc<dyn QueuePort>,
    scrapers: Vec<Arc<dyn ScraperPort>>,
}

impl SearchService {
    pub fn new(
        cache: Arc<dyn CachePort>,
        queue: Arc<dyn QueuePort>,
        scrapers: Vec<Arc<dyn ScraperPort>>,
    ) -> Self {
        Self {
            cache,
            queue,
            scrapers,
        }
    }

    pub async fn search_products(&self, query: &str, limit: Option<usize>) -> DomainResult<Vec<Product>> {
        // Try to get products from cache first
        if let Ok(cached_products) = self.cache.get_products(query).await {
            info!("Found {} products in cache for query: {}", cached_products.len(), query);
            return Ok(cached_products);
        }

        // Enqueue scraping job for background processing
        self.queue.enqueue_scrape_job(query).await?;
        info!("Enqueued scraping job for query: {}", query);

        // Do immediate scraping for the request
        let mut all_products = Vec::new();
        for scraper in &self.scrapers {
            match scraper.scrape_products(query, limit).await {
                Ok(products) => {
                    info!("Found {} products from scraper", products.len());
                    all_products.extend(products);
                }
                Err(e) => warn!("Error scraping products: {}", e),
            }
        }

        // Cache the results
        if !all_products.is_empty() {
            if let Err(e) = self.cache.cache_products(query, &all_products).await {
                warn!("Failed to cache products: {}", e);
            }
        }

        Ok(all_products)
    }

    pub async fn search_store_products(&self, store: Store, query: &str, limit: Option<usize>) -> DomainResult<Vec<Product>> {
        // Try to get products from cache first
        if let Ok(cached_products) = self.cache.get_products(query).await {
            let store_products: Vec<_> = cached_products
                .into_iter()
                .filter(|p| p.store() == store)
                .collect();
            
            if !store_products.is_empty() {
                info!("Found {} products in cache for store {} and query: {}", 
                    store_products.len(), store, query);
                return Ok(store_products);
            }
        }

        // Find the appropriate scraper for the store
        for scraper in &self.scrapers {
            if scraper.get_store() == store {
                let products = scraper.scrape_products(query, limit).await?;
                
                // Cache the results
                if !products.is_empty() {
                    if let Err(e) = self.cache.cache_products(query, &products).await {
                        warn!("Failed to cache products: {}", e);
                    }
                }

                return Ok(products);
            }
        }

        Ok(Vec::new())
    }

    pub async fn get_product_details(&self, url: &str) -> DomainResult<Product> {
        for scraper in &self.scrapers {
            if scraper.can_handle_url(url) {
                return scraper.get_product_details(url).await;
            }
        }
        
        Err(DomainError::not_found("No scraper found for URL"))
    }
} 