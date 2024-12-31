use async_trait::async_trait;
use crate::domain::models::{Product, Store, DomainError};

#[async_trait]
pub trait ProductSearchPort: Send + Sync {
    /// Search for products across all stores
    async fn search_products(&self, query: &str) -> Result<Vec<Product>, DomainError>;
    
    /// Search for products in a specific store
    async fn search_store_products(&self, store: Store, query: &str) -> Result<Vec<Product>, DomainError>;
    
    /// Get detailed information about a specific product
    async fn get_product_details(&self, url: &str) -> Result<Product, DomainError>;
} 