use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::domain::models::{Store, DomainError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeJob {
    pub query: String,
    pub store: Option<Store>,
    pub priority: JobPriority,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobPriority {
    High,
    Normal,
    Low,
}

#[async_trait]
pub trait QueuePort: Send + Sync {
    /// Enqueue a new scraping job
    async fn enqueue_job(&self, job: ScrapeJob) -> Result<(), DomainError>;
    
    /// Process the next available job
    async fn process_next_job(&self) -> Result<Option<ScrapeJob>, DomainError>;
    
    /// Get the number of pending jobs
    async fn pending_jobs_count(&self) -> Result<usize, DomainError>;
    
    /// Clear all pending jobs
    async fn clear_jobs(&self) -> Result<(), DomainError>;
} 