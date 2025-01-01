use async_trait::async_trait;
use crate::domain::models::{DomainResult, Store};

#[async_trait]
pub trait QueuePort: Send + Sync {
    /// Enqueue a job to scrape all stores
    async fn enqueue_scrape_job(&self, query: &str) -> DomainResult<()>;
    
    /// Enqueue a job to scrape a specific store
    async fn enqueue_store_scrape_job(&self, query: &str, store: &Store) -> DomainResult<()>;
    
    /// Process a job for all stores
    async fn process_scrape_job(&self, query: &str) -> DomainResult<()>;
    
    /// Process a job for a specific store
    async fn process_store_scrape_job(&self, query: &str, store: &Store) -> DomainResult<()>;
} 