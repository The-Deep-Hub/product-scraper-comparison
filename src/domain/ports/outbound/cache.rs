use async_trait::async_trait;

use crate::domain::models::{Product, DomainResult};

#[async_trait]
pub trait CachePort: Send + Sync {
    async fn get_products(&self, query: &str) -> DomainResult<Vec<Product>>;
    
    async fn cache_products(&self, query: &str, products: &[Product]) -> DomainResult<()>;
    
    async fn invalidate(&self, query: &str) -> DomainResult<()>;
} 