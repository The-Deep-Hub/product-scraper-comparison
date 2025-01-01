use async_trait::async_trait;
use crate::domain::models::{Product, DomainError, Store};

#[async_trait]
pub trait CachePort: Send + Sync {
    /// Get all products for all stores for a given query
    async fn get_products(&self, query: &str) -> Result<Vec<Product>, DomainError>;
    
    /// Get products for a specific store and query
    async fn get_store_products(&self, query: &str, store: &Store) -> Result<Vec<Product>, DomainError>;
    
    /// Cache products for all stores
    async fn cache_products(&self, query: &str, products: &[Product]) -> Result<(), DomainError>;
    
    /// Cache products for a specific store
    async fn cache_store_products(&self, query: &str, store: &Store, products: &[Product]) -> Result<(), DomainError>;
    
    /// Set a generic value in the cache
    async fn set_value(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), DomainError>;
    
    /// Get a generic value from the cache
    async fn get_value(&self, key: &str) -> Result<Option<String>, DomainError>;
} 