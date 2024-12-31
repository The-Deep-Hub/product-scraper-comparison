use async_trait::async_trait;

use crate::domain::models::DomainResult;

#[async_trait]
pub trait QueuePort: Send + Sync {
    async fn enqueue_scrape_job(&self, query: &str) -> DomainResult<()>;
    
    async fn process_scrape_job(&self, query: &str) -> DomainResult<()>;
} 