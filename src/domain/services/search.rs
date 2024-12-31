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

        // Enqueue search job
        self.queue.enqueue_scrape_job(query).await?;

        // Process the job
        self.queue.process_scrape_job(query).await?;

        // Get results from cache
        self.cache.get_products(query).await
    }

    async fn search_store_products(&self, store: Store, query: &str) -> DomainResult<Vec<Product>> {
        // Try to get from cache first
        if let Ok(products) = self.cache.get_products(query).await {
            let store_products: Vec<Product> = products
                .into_iter()
                .filter(|p| p.store() == store)
                .collect();
            
            if !store_products.is_empty() {
                return Ok(store_products);
            }
        }

        // Enqueue search job
        self.queue.enqueue_scrape_job(query).await?;

        // Process the job
        self.queue.process_scrape_job(query).await?;

        // Get results from cache and filter by store
        let products = self.cache.get_products(query).await?;
        Ok(products
            .into_iter()
            .filter(|p| p.store() == store)
            .collect())
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