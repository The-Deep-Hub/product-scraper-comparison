use async_trait::async_trait;
use crate::domain::models::{Product, Store, DomainError};

#[async_trait]
pub trait ScraperPort: Send + Sync {
    /// Get the store this scraper is responsible for
    fn get_store(&self) -> Store;
    
    /// Scrape products from the store
    async fn scrape_products(&self, query: &str, limit: Option<usize>) -> Result<Vec<Product>, DomainError>;
    
    /// Get detailed product information
    async fn get_product_details(&self, url: &str) -> Result<Product, DomainError>;
    
    /// Check if this scraper can handle a given URL
    fn can_handle_url(&self, url: &str) -> bool;
} 