use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::domain::{
    events::{DomainEvent, EventHandler},
    models::{Store, DomainResult},
};

#[derive(Debug, Default, Clone)]
struct Metrics {
    searches_requested: u64,
    searches_completed: u64,
    products_scraped: HashMap<Store, u64>,
    scraping_failures: HashMap<Store, u64>,
    rate_limits_hit: HashMap<Store, u64>,
    parsing_errors: HashMap<Store, u64>,
    cache_hits: HashMap<Store, u64>,
    cache_misses: HashMap<Store, u64>,
    jobs_enqueued: HashMap<Store, u64>,
    jobs_completed: HashMap<Store, u64>,
    jobs_failed: HashMap<Store, u64>,
}

pub struct MetricsHandler {
    metrics: Arc<RwLock<Metrics>>,
}

impl MetricsHandler {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics::default())),
        }
    }

    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.read().await.clone()
    }
}

#[async_trait]
impl EventHandler for MetricsHandler {
    async fn handle(&self, event: DomainEvent) -> DomainResult<()> {
        let mut metrics = self.metrics.write().await;
        
        match &event {
            DomainEvent::SearchRequested { .. } => {
                metrics.searches_requested += 1;
            }
            DomainEvent::SearchCompleted { total_products, .. } => {
                metrics.searches_completed += 1;
                info!("Search completed with {} products", total_products);
            }
            DomainEvent::ProductsScraped { store, products, .. } => {
                *metrics.products_scraped.entry(store.clone()).or_default() += 1;
                info!("Scraped {} products from {}", products.len(), store);
            }
            DomainEvent::ScrapingFailed { store, error, .. } => {
                *metrics.scraping_failures.entry(store.clone()).or_default() += 1;
                warn!("Scraping failed for {}: {}", store, error);
            }
            DomainEvent::RateLimitReached { store, retry_after, .. } => {
                *metrics.rate_limits_hit.entry(store.clone()).or_default() += 1;
                warn!("Rate limit reached for {}. Retry after {} seconds", store, retry_after);
            }
            DomainEvent::ParsingError { store, url, error, .. } => {
                *metrics.parsing_errors.entry(store.clone()).or_default() += 1;
                warn!("Parsing error for {} at {}: {}", store, url, error);
            }
            DomainEvent::CacheHit { store, query, .. } => {
                *metrics.cache_hits.entry(store.clone()).or_default() += 1;
                info!("Cache hit for {} query: {}", store, query);
            }
            DomainEvent::CacheMiss { store, query, .. } => {
                *metrics.cache_misses.entry(store.clone()).or_default() += 1;
                info!("Cache miss for {} query: {}", store, query);
            }
            DomainEvent::JobEnqueued { store, query, .. } => {
                *metrics.jobs_enqueued.entry(store.clone()).or_default() += 1;
                info!("Job enqueued for {} query: {}", store, query);
            }
            DomainEvent::JobCompleted { store, products_count, .. } => {
                *metrics.jobs_completed.entry(store.clone()).or_default() += 1;
                info!("Job completed for {} with {} products", store, products_count);
            }
            DomainEvent::JobFailed { store, error, .. } => {
                *metrics.jobs_failed.entry(store.clone()).or_default() += 1;
                warn!("Job failed for {}: {}", store, error);
            }
            _ => {}
        }

        Ok(())
    }
}