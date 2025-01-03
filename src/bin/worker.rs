use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use tracing_subscriber::{fmt, EnvFilter};

use rust_scraper::{
    adapters::{
        outbound::{
            cache::RedisAdapter,
            queue::RabbitMQAdapter,
            http::ZyteAdapter,
            scrapers::{LeroyScraper, BauhausScraper, BricodepotScraper},
            events::{
                InMemoryEventPublisher,
                handlers::ProductCacheHandler,
            },
        },
        inbound::worker::processor::ScrapeTask,
    },
    domain::{
        services::scraper::ScraperService,
        ports::outbound::{ScraperPort, EventPublisherPort, QueuePort, CachePort},
        models::Store,
        events::{DomainEvent, EventMetadata},
        config::QueueConfig,
    },
    config::builder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    fmt()
        .with_env_filter(EnvFilter::new("rust_scraper=info,worker=info"))
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_file(false)
        .with_level(true)
        .init();

    info!("Starting worker service...");
    let config = builder::new().expect("Failed to load configuration");

    // Initialize adapters
    let cache_adapter = Arc::new(RedisAdapter::new(Box::new(config.cache.clone())).await?);
    let queue_adapter = Arc::new(RabbitMQAdapter::new(config.queue.connection_url(), config.queue.queue_name()).await?);
    let zyte_adapter = Arc::new(ZyteAdapter::new(config.http_client.api_key.clone().expect("Zyte API key must be configured")));

    // Initialize scrapers
    let scrapers = vec![
        Arc::new(LeroyScraper::new(Box::new((*zyte_adapter).clone())).expect("Failed to create Leroy scraper")) as Arc<dyn ScraperPort>,
        Arc::new(BauhausScraper::new(Box::new((*zyte_adapter).clone()))) as Arc<dyn ScraperPort>,
        Arc::new(BricodepotScraper::new(Box::new((*zyte_adapter).clone()))) as Arc<dyn ScraperPort>,
    ];

    // Initialize event system
    let event_publisher: Arc<dyn EventPublisherPort> = Arc::new(InMemoryEventPublisher::new());
    let cache_handler = ProductCacheHandler::new(cache_adapter.clone(), event_publisher.clone());
    event_publisher.register_handler(Box::new(cache_handler)).await?;

    let scraper_service = Arc::new(ScraperService::new(scrapers));
    let (tx, mut rx) = mpsc::channel::<ScrapeTask>(100);

    // Start a single consumer for each store
    for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
        let store_queue = queue_adapter.clone();
        let store_tx = tx.clone();
        let store_clone = store.clone();
        
        tokio::spawn(async move {
            loop {
                if let Err(e) = store_queue.consume_messages(&store_clone, store_tx.clone()).await {
                    error!("Consumer error for {}: {}", store_clone, e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    }

    info!("Worker ready");

    // Main processing loop
    while let Some(task) = rx.recv().await {
        match scraper_service.scrape_store_products(task.store, &task.query, task.num_products).await {
            Ok(products) => {
                info!("Scraped {} products from {} for '{}'", products.len(), task.store, task.query);
                
                if let Err(e) = cache_adapter.cache_store_products(&task.query, &task.store, &products).await {
                    error!("Cache error for {}: {}", task.store, e);
                }

                if let Err(e) = event_publisher
                    .publish(DomainEvent::ProductsScraped {
                        metadata: EventMetadata::new(),
                        query: task.query,
                        products,
                        store: task.store,
                    })
                    .await
                {
                    error!("Event publish error: {}", e);
                }
            }
            Err(e) => error!("Scrape error for {}: {}", task.store, e),
        }
    }

    Ok(())
} 