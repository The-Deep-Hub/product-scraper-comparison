use async_trait::async_trait;
use crate::domain::{
    models::{Product, Store, DomainResult},
    ports::{
        outbound::{CachePort, QueuePort, HttpClientPort},
        inbound::ProductSearchPort,
    },
};

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
}

#[async_trait]
impl<C, Q, H> ProductSearchPort for SearchService<C, Q, H>
where
    C: CachePort + Clone,
    Q: QueuePort + Clone,
    H: HttpClientPort + Clone,
{
    async fn search_products(&self, query: &str) -> DomainResult<Vec<Product>> {
        // Try to get from cache first
        if let Ok(products) = self.cache.get_products(query).await {
            if !products.is_empty() {
                return Ok(products);
            }
        }

        // Enqueue search job for all stores
        self.queue.enqueue_scrape_job(query).await?;

        // Process the job
        self.queue.process_scrape_job(query).await?;

        // Get results from cache
        self.cache.get_products(query).await
    }

    async fn search_store_products(&self, store: Store, query: &str) -> DomainResult<Vec<Product>> {
        // Try to get from store-specific cache first
        if let Ok(products) = self.cache.get_store_products(query, &store).await {
            if !products.is_empty() {
                return Ok(products);
            }
        }

        // Enqueue search job for specific store
        self.queue.enqueue_store_scrape_job(query, &store).await?;

        // Process the job for specific store
        self.queue.process_store_scrape_job(query, &store).await?;

        // Get results from store-specific cache
        self.cache.get_store_products(query, &store).await
    }

    async fn get_product_details(&self, url: &str) -> DomainResult<Product> {
        // Try to get from cache first
        if let Ok(products) = self.cache.get_products(url).await {
            if let Some(product) = products.into_iter().next() {
                return Ok(product);
            }
        }

        // Fetch and parse product details
        let html = self.http.get_rendered_html(url).await?;
        
        // TODO: Implement product parsing
        todo!("Implement product parsing")
    }
} 