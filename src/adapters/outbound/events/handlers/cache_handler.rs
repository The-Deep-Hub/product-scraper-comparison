use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, error};

use crate::domain::{
    events::{DomainEvent, EventHandler, EventMetadata},
    models::DomainResult,
    ports::outbound::{CachePort, EventPublisherPort},
};

/// Handler that automatically caches products when they're scraped
pub struct ProductCacheHandler {
    cache: Arc<dyn CachePort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl ProductCacheHandler {
    pub fn new(cache: Arc<dyn CachePort>, event_publisher: Arc<dyn EventPublisherPort>) -> Self {
        Self { 
            cache,
            event_publisher,
        }
    }
}

#[async_trait]
impl EventHandler for ProductCacheHandler {
    async fn handle(&self, event: DomainEvent) -> DomainResult<()> {
        match event {
            DomainEvent::ProductsScraped { query, products, store, metadata } => {
                info!(
                    "Caching {} products for query '{}' in store {} (event_id: {})",
                    products.len(),
                    query,
                    store,
                    metadata.id
                );

                // Cache the products
                self.cache.cache_products(&query, &products).await?;

                // Emit ProductsCached event
                self.event_publisher
                    .publish(DomainEvent::ProductsCached {
                        metadata: EventMetadata::new(),
                        query,
                        products,
                        store,
                    })
                    .await?;

                Ok(())
            }
            // Ignore other events
            _ => Ok(()),
        }
    }
} 