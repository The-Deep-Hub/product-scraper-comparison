use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tracing::{info, warn};

use crate::domain::models::{Product, Store, DomainError};
use crate::domain::ports::{
    ProductSearchPort,
    CachePort,
    ScraperPort,
    QueuePort,
    outbound::{ScrapeJob, JobPriority},
};

pub struct ProductSearchService {
    cache: Arc<dyn CachePort>,
    scrapers: Vec<Arc<dyn ScraperPort>>,
    queue: Arc<dyn QueuePort>,
    cache_ttl: Duration,
}

impl ProductSearchService {
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

    async fn get_scraper_for_url(&self, url: &str) -> Option<Arc<dyn ScraperPort>> {
        self.scrapers
            .iter()
            .find(|scraper| scraper.can_handle_url(url))
            .cloned()
    }

    async fn get_scraper_for_store(&self, store: Store) -> Option<Arc<dyn ScraperPort>> {
        self.scrapers
            .iter()
            .find(|scraper| scraper.get_store() == store)
            .cloned()
    }

    async fn enqueue_background_scrape(&self, query: &str, store: Option<Store>) -> Result<(), DomainError> {
        let job = ScrapeJob {
            query: query.to_string(),
            store,
            priority: JobPriority::Normal,
        };
        self.queue.enqueue_job(job).await
    }
}

#[async_trait]
impl ProductSearchPort for ProductSearchService {
    async fn search_products(&self, query: &str) -> Result<Vec<Product>, DomainError> {
        // Try to get from cache first
        if let Ok(Some(products)) = self.cache.get_products(query).await {
            info!("Found {} products in cache for query: {}", products.len(), query);
            return Ok(products);
        }

        // Enqueue background job to refresh cache
        if let Err(e) = self.enqueue_background_scrape(query, None).await {
            warn!("Failed to enqueue background scrape: {}", e);
        }

        // Perform immediate scrape
        let mut all_products = Vec::new();
        for scraper in &self.scrapers {
            match scraper.scrape_products(query, Some(10)).await {
                Ok(products) => {
                    info!(
                        "Found {} products from {} for query: {}",
                        products.len(),
                        scraper.get_store(),
                        query
                    );
                    all_products.extend(products);
                }
                Err(e) => {
                    warn!(
                        "Failed to scrape products from {}: {}",
                        scraper.get_store(),
                        e
                    );
                }
            }
        }

        // Cache results
        if !all_products.is_empty() {
            if let Err(e) = self.cache.cache_products(query, all_products.clone(), self.cache_ttl).await {
                warn!("Failed to cache products: {}", e);
            }
        }

        Ok(all_products)
    }

    async fn search_store_products(&self, store: Store, query: &str) -> Result<Vec<Product>, DomainError> {
        // Try to get from cache first
        if let Ok(Some(products)) = self.cache.get_products(query).await {
            let store_products: Vec<Product> = products
                .into_iter()
                .filter(|p| p.store() == store)
                .collect();

            if !store_products.is_empty() {
                info!("Found {} products in cache for store {} and query: {}", 
                    store_products.len(), store, query);
                return Ok(store_products);
            }
        }

        // Enqueue background job to refresh cache
        if let Err(e) = self.enqueue_background_scrape(query, Some(store)).await {
            warn!("Failed to enqueue background scrape: {}", e);
        }

        // Get store-specific scraper
        let scraper = self.get_scraper_for_store(store).await
            .ok_or_else(|| DomainError::not_found(format!("No scraper found for store: {}", store)))?;

        // Perform immediate scrape
        let products = scraper.scrape_products(query, Some(10)).await?;
        
        // Cache results
        if !products.is_empty() {
            if let Err(e) = self.cache.cache_products(query, products.clone(), self.cache_ttl).await {
                warn!("Failed to cache products: {}", e);
            }
        }

        Ok(products)
    }

    async fn get_product_details(&self, url: &str) -> Result<Product, DomainError> {
        let scraper = self.get_scraper_for_url(url).await
            .ok_or_else(|| DomainError::not_found(format!("No scraper found for URL: {}", url)))?;

        scraper.get_product_details(url).await
    }
} 