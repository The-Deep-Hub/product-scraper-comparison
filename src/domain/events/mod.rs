use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::domain::models::{Product, Store, DomainResult};

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
    // Search Events
    SearchRequested {
        metadata: EventMetadata,
        query: String,
        stores: Vec<Store>,
    },
    SearchCompleted {
        metadata: EventMetadata,
        query: String,
        total_products: usize,
    },

    // Scraping Events
    ProductsScraped {
        metadata: EventMetadata,
        query: String,
        products: Vec<Product>,
        store: Store,
    },
    ScrapingFailed {
        metadata: EventMetadata,
        query: String,
        store: Store,
        error: String,
    },
    RateLimitReached {
        metadata: EventMetadata,
        store: Store,
        retry_after: u64,
    },
    ParsingError {
        metadata: EventMetadata,
        store: Store,
        url: String,
        error: String,
    },

    // Cache Events
    ProductsCached {
        metadata: EventMetadata,
        query: String,
        products: Vec<Product>,
        store: Store,
    },
    CacheHit {
        metadata: EventMetadata,
        query: String,
        store: Store,
    },
    CacheMiss {
        metadata: EventMetadata,
        query: String,
        store: Store,
    },

    // Queue Events
    JobEnqueued {
        metadata: EventMetadata,
        query: String,
        store: Store,
    },
    JobStarted {
        metadata: EventMetadata,
        query: String,
        store: Store,
    },
    JobCompleted {
        metadata: EventMetadata,
        query: String,
        store: Store,
        products_count: usize,
    },
    JobFailed {
        metadata: EventMetadata,
        query: String,
        store: Store,
        error: String,
    },
}

impl DomainEvent {
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            Self::SearchRequested { metadata, .. } => metadata,
            Self::SearchCompleted { metadata, .. } => metadata,
            Self::ProductsScraped { metadata, .. } => metadata,
            Self::ScrapingFailed { metadata, .. } => metadata,
            Self::RateLimitReached { metadata, .. } => metadata,
            Self::ParsingError { metadata, .. } => metadata,
            Self::ProductsCached { metadata, .. } => metadata,
            Self::CacheHit { metadata, .. } => metadata,
            Self::CacheMiss { metadata, .. } => metadata,
            Self::JobEnqueued { metadata, .. } => metadata,
            Self::JobStarted { metadata, .. } => metadata,
            Self::JobCompleted { metadata, .. } => metadata,
            Self::JobFailed { metadata, .. } => metadata,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            Self::SearchRequested { .. } => "SearchRequested",
            Self::SearchCompleted { .. } => "SearchCompleted",
            Self::ProductsScraped { .. } => "ProductsScraped",
            Self::ScrapingFailed { .. } => "ScrapingFailed",
            Self::RateLimitReached { .. } => "RateLimitReached",
            Self::ParsingError { .. } => "ParsingError",
            Self::ProductsCached { .. } => "ProductsCached",
            Self::CacheHit { .. } => "CacheHit",
            Self::CacheMiss { .. } => "CacheMiss",
            Self::JobEnqueued { .. } => "JobEnqueued",
            Self::JobStarted { .. } => "JobStarted",
            Self::JobCompleted { .. } => "JobCompleted",
            Self::JobFailed { .. } => "JobFailed",
        }
    }
}

/// Trait for handling domain events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a domain event
    async fn handle(&self, event: DomainEvent) -> DomainResult<()>;
} 