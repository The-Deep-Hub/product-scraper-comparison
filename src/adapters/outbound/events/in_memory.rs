use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::domain::{
    events::{DomainEvent, EventHandler},
    models::DomainResult,
    ports::outbound::EventPublisherPort,
};

/// In-memory event publisher for testing and development
pub struct InMemoryEventPublisher {
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
}

impl InMemoryEventPublisher {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisherPort for InMemoryEventPublisher {
    async fn register_handler(&self, handler: Box<dyn EventHandler>) -> DomainResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
        Ok(())
    }

    async fn publish(&self, event: DomainEvent) -> DomainResult<()> {
        info!(
            "Publishing event: {} (id: {})", 
            event.event_type(), 
            event.metadata().id
        );

        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            if let Err(e) = handler.handle(event.clone()).await {
                error!(
                    "Error handling event {} (id: {}): {}", 
                    event.event_type(), 
                    event.metadata().id,
                    e
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::domain::events::EventMetadata;

    struct TestHandler {
        received: Arc<Mutex<Vec<DomainEvent>>>,
    }

    #[async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, event: DomainEvent) -> DomainResult<()> {
            let mut received = self.received.lock().await;
            received.push(event);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_publish_and_handle() {
        let publisher = InMemoryEventPublisher::new();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        
        let handler = TestHandler {
            received: Arc::clone(&received_events),
        };

        publisher.register_handler(Box::new(handler)).await.unwrap();

        let event = DomainEvent::ScrapeJobEnqueued {
            metadata: EventMetadata::new(),
            query: "test".to_string(),
        };

        publisher.publish(event.clone()).await.unwrap();

        let received = received_events.lock().await;
        assert_eq!(received.len(), 1);
        
        match &received[0] {
            DomainEvent::ScrapeJobEnqueued { query, .. } => {
                assert_eq!(query, "test");
            }
            _ => panic!("Wrong event type received"),
        }
    }
} 