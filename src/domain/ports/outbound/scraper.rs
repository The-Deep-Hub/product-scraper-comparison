use async_trait::async_trait;

use crate::domain::models::{Product, Store, DomainResult};

#[async_trait]
pub trait ScraperPort: Send + Sync {
    fn get_store(&self) -> Store;
    
    async fn scrape_products(&self, query: &str, limit: Option<usize>) -> DomainResult<Vec<Product>>;
    
    async fn get_product_details(&self, url: &str) -> DomainResult<Product>;
    
    fn can_handle_url(&self, url: &str) -> bool;
} 