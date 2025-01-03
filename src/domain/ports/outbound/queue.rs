use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::{
    domain::models::{DomainResult, Store},
    adapters::inbound::worker::processor::ScrapeTask,
};

#[async_trait]
pub trait QueuePort: Send + Sync {
    /// Enqueue a job to scrape all stores
    async fn enqueue_scrape_job(&self, task_id: &str, query: &str, store: Store, num_products: Option<usize>) -> DomainResult<()>;
    
    /// Process a job for all stores
    async fn process_scrape_job(&self, query: &str, num_products: Option<usize>) -> DomainResult<()>;
    
    /// Process a job for a specific store, returns Some(query) if a message was found and processed
    async fn process_store_scrape_job(&self, query: &str, store: &Store, num_products: Option<usize>) -> DomainResult<Option<String>>;

    /// Get a value from the queue storage
    async fn get_value(&self, key: &str) -> DomainResult<Option<String>>;

    /// Consume messages for a specific store
    async fn consume_messages(&self, store: &Store, tx: mpsc::Sender<ScrapeTask>) -> DomainResult<()>;
} 