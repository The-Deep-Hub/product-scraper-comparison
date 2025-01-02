use std::sync::Arc;
use std::time::Duration;
use mockall::predicate::*;
use async_trait::async_trait;
use crate::domain::{
    models::{Product, Store, DomainError},
    ports::{CachePort, ScraperPort, QueuePort},
    ports::outbound::ScrapeJob,
};
use super::{ProductSearchService, ScraperWorkerService};

// Import the mocks from our domain tests
use crate::domain::tests::{MockCachePort, MockScraperPort};

mock! {
    QueuePort {}

    #[async_trait]
    impl QueuePort for QueuePort {
        async fn enqueue_job(&self, job: ScrapeJob) -> Result<(), DomainError>;
        async fn process_next_job(&self) -> Result<Option<ScrapeJob>, DomainError>;
        async fn pending_jobs_count(&self) -> Result<usize, DomainError>;
        async fn clear_jobs(&self) -> Result<(), DomainError>;
    }
}

#[tokio::test]
async fn test_product_search_service_cache_hit() {
    let mut cache = MockCachePort::new();
    let scraper = MockScraperPort::new();
    let queue = MockQueuePort::new();
    
    let cached_products = vec![
        Product::new(
            "Cached Product".to_string(),
            "Description".to_string(),
            10.99,
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        ).unwrap()
    ];

    cache
        .expect_get_products()
        .with(eq("test"))
        .returning(move |_| Ok(Some(cached_products.clone())));

    let service = ProductSearchService::new(
        Arc::new(cache),
        vec![Arc::new(scraper)],
        Arc::new(queue),
        Duration::from_secs(3600),
    );

    let results = service.search_products("test").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name(), "Cached Product");
}

#[tokio::test]
async fn test_product_search_service_cache_miss() {
    let mut cache = MockCachePort::new();
    let mut scraper = MockScraperPort::new();
    let mut queue = MockQueuePort::new();
    
    let scraped_products = vec![
        Product::new(
            "Scraped Product".to_string(),
            "Description".to_string(),
            10.99,
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        ).unwrap()
    ];

    cache
        .expect_get_products()
        .with(eq("test"))
        .returning(|_| Ok(None));

    cache
        .expect_cache_products()
        .returning(|_, _, _| Ok(()));

    scraper
        .expect_get_store()
        .returning(|| Store::LeroyMerlin);

    scraper
        .expect_scrape_products()
        .returning(move |_, _| Ok(scraped_products.clone()));

    queue
        .expect_enqueue_job()
        .returning(|_| Ok(()));

    let service = ProductSearchService::new(
        Arc::new(cache),
        vec![Arc::new(scraper)],
        Arc::new(queue),
        Duration::from_secs(3600),
    );

    let results = service.search_products("test").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name(), "Scraped Product");
}

#[tokio::test]
async fn test_product_search_service_store_specific() {
    let mut cache = MockCachePort::new();
    let mut scraper = MockScraperPort::new();
    let mut queue = MockQueuePort::new();
    
    let store_products = vec![
        Product::new(
            "Store Product".to_string(),
            "Description".to_string(),
            10.99,
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        ).unwrap()
    ];

    cache
        .expect_get_products()
        .with(eq("test"))
        .returning(|_| Ok(None));

    cache
        .expect_cache_products()
        .returning(|_, _, _| Ok(()));

    scraper
        .expect_get_store()
        .returning(|| Store::LeroyMerlin);

    scraper
        .expect_scrape_products()
        .returning(move |_, _| Ok(store_products.clone()));

    queue
        .expect_enqueue_job()
        .returning(|_| Ok(()));

    let service = ProductSearchService::new(
        Arc::new(cache),
        vec![Arc::new(scraper)],
        Arc::new(queue),
        Duration::from_secs(3600),
    );

    let results = service.search_store_products(Store::LeroyMerlin, "test").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name(), "Store Product");
} 