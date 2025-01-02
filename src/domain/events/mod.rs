use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::domain::models::{Product, DomainResult};

/// Metadata attached to all domain events
#[derive(Debug, Clone)]
pub struct EventMetadata {
    /// Unique identifier for the event
    pub id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Optional ID to correlate related events
    pub correlation_id: Option<String>,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            correlation_id: None,
        }
    }

    pub fn with_correlation(correlation_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            correlation_id: Some(correlation_id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DomainEvent {
    ProductsScraped {
        metadata: EventMetadata,
        query: String,
        products: Vec<Product>,
    },
    ProductsCached {
        metadata: EventMetadata,
        query: String,
        products: Vec<Product>,
    },
    ScrapeJobEnqueued {
        metadata: EventMetadata,
        query: String,
    },
    ScrapeJobCompleted {
        metadata: EventMetadata,
        query: String,
        products: Vec<Product>,
    },
}

impl DomainEvent {
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            Self::ProductsScraped { metadata, .. } => metadata,
            Self::ProductsCached { metadata, .. } => metadata,
            Self::ScrapeJobEnqueued { metadata, .. } => metadata,
            Self::ScrapeJobCompleted { metadata, .. } => metadata,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            Self::ProductsScraped { .. } => "ProductsScraped",
            Self::ProductsCached { .. } => "ProductsCached",
            Self::ScrapeJobEnqueued { .. } => "ScrapeJobEnqueued",
            Self::ScrapeJobCompleted { .. } => "ScrapeJobCompleted",
        }
    }
}

/// Trait for handling domain events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a domain event
    async fn handle(&self, event: DomainEvent) -> DomainResult<()>;
} 