use async_trait::async_trait;
use std::time::Duration;
use crate::domain::models::{Product, DomainError};

#[async_trait]
pub trait CachePort: Send + Sync {
    /// Get cached products for a query
    async fn get_products(&self, query: &str) -> Result<Option<Vec<Product>>, DomainError>;
    
    /// Cache products for a query with TTL
    async fn cache_products(&self, query: &str, products: Vec<Product>, ttl: Duration) -> Result<(), DomainError>;
    
    /// Invalidate cache for a query
    async fn invalidate(&self, query: &str) -> Result<(), DomainError>;
    
    /// Check if a query is cached
    async fn is_cached(&self, query: &str) -> Result<bool, DomainError>;
} 