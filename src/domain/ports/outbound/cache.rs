use async_trait::async_trait;
use crate::domain::models::{Product, DomainError};

#[async_trait]
pub trait CachePort: Send + Sync {
    async fn get_products(&self, key: &str) -> Result<Vec<Product>, DomainError>;
    async fn cache_products(&self, key: &str, products: &[Product]) -> Result<(), DomainError>;
    async fn set_value(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), DomainError>;
    async fn get_value(&self, key: &str) -> Result<Option<String>, DomainError>;
} 