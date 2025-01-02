use async_trait::async_trait;
use crate::domain::{
    events::{DomainEvent, EventHandler},
    models::DomainResult,
};

/// Port for publishing domain events
#[async_trait]
pub trait EventPublisherPort: Send + Sync {
    /// Register an event handler
    async fn register_handler(&self, handler: Box<dyn EventHandler>) -> DomainResult<()>;

    /// Publish a domain event to all registered handlers
    async fn publish(&self, event: DomainEvent) -> DomainResult<()>;

    /// Publish multiple domain events
    async fn publish_all(&self, events: Vec<DomainEvent>) -> DomainResult<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
} 