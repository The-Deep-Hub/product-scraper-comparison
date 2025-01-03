use async_trait::async_trait;
use crate::domain::{
    models::{Product, Store, DomainResult, DomainError},
    ports::{
        outbound::{CachePort, QueuePort, HttpClientPort, ScraperPort},
        inbound::ProductSearchPort,
    },
};
use uuid::Uuid;
use std::sync::Arc;

#[derive(Clone)]
pub struct SearchService<C, Q, H>
where
    C: CachePort + Clone,
    Q: QueuePort + Clone,
    H: HttpClientPort + Clone,
{
    cache: C,
    queue: Q,
    http: H,
}

impl<C, Q, H> SearchService<C, Q, H>
where
    C: CachePort + Clone,
    Q: QueuePort + Clone,
    H: HttpClientPort + Clone,
{
    pub fn new(cache: C, queue: Q, http: H) -> Self {
        Self { cache, queue, http }
    }

    async fn get_scraper_for_url(&self, _url: &str) -> Option<Arc<dyn ScraperPort>> {
        // TODO: Implement this method
        // For now, we'll return None since we don't have access to scrapers in this service
        None
    }
}

#[async_trait]
impl<C, Q, H> ProductSearchPort for SearchService<C, Q, H>
where
    C: CachePort + Clone,
    Q: QueuePort + Clone,
    H: HttpClientPort + Clone,
{
    async fn search_products(&self, query: &str, num_products: usize) -> DomainResult<Vec<Product>> {
        // Try to get from cache first
        if let Ok(products) = self.cache.get_products(query).await {
            if !products.is_empty() {
                return Ok(products);
            }
        }

        // Enqueue background job for all stores
        self.queue.process_scrape_job(query, Some(num_products)).await?;

        // Get results from cache
        self.cache.get_products(query).await
    }

    async fn search_store_products(&self, store: Store, query: &str, num_products: usize) -> DomainResult<Vec<Product>> {
        // Try to get from store-specific cache first
        if let Ok(products) = self.cache.get_store_products(query, &store).await {
            if !products.is_empty() {
                return Ok(products);
            }
        }

        // Enqueue background job for specific store
        let task_id = Uuid::new_v4().to_string();
        self.queue.enqueue_scrape_job(&task_id, query, store.clone(), Some(num_products)).await?;

        // Process the job for specific store
        self.queue.process_store_scrape_job(query, &store, Some(num_products)).await?;

        // Get results from store-specific cache
        self.cache.get_store_products(query, &store).await
    }

    async fn get_product_details(&self, url: &str) -> DomainResult<Product> {
        // This method doesn't need num_products parameter
        let scraper = self.get_scraper_for_url(url).await
            .ok_or_else(|| DomainError::not_found(format!("No scraper found for URL: {}", url)))?;

        scraper.get_product_details(url).await
    }
} 