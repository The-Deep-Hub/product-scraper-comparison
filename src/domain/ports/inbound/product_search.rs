use async_trait::async_trait;
use crate::domain::models::{Product, Store, DomainResult};

#[async_trait]
pub trait ProductSearchPort: Send + Sync + Clone {
    /// Search for products across all stores
    async fn search_products(&self, query: &str, num_products: usize) -> DomainResult<Vec<Product>>;
    
    /// Search for products in a specific store
    async fn search_store_products(&self, store: Store, query: &str, num_products: usize) -> DomainResult<Vec<Product>>;
    
    /// Get detailed information about a specific product
    async fn get_product_details(&self, url: &str) -> DomainResult<Product>;
} 