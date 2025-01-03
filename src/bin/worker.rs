use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error};
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
        config::QueueConfig,
        models::Store,
        events::{DomainEvent, EventMetadata},
    },
    config::builder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with better configuration
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("rust_scraper=debug,worker=debug"))
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_level(true)
        .init();

    info!("Starting worker service...");

    // Load configuration
    let config = builder::new().expect("Failed to load configuration");
    info!("Configuration loaded successfully");

    // Initialize adapters
    info!("Initializing adapters...");
    let cache_adapter = Arc::new(
        RedisAdapter::new(Box::new(config.cache.clone()))
            .await
            .expect("Failed to create Redis adapter")
    );
    info!("Redis adapter initialized");

    let queue_adapter = Arc::new(
        RabbitMQAdapter::new(
            config.queue.connection_url(),
            config.queue.queue_name(),
        )
        .await
        .expect("Failed to create RabbitMQ adapter")
    );
    info!("RabbitMQ adapter initialized");

    let zyte_adapter = Arc::new(ZyteAdapter::new(
        config.http_client.api_key.clone()
            .expect("Zyte API key must be configured")
    ));
    info!("Zyte adapter initialized");

    // Initialize scrapers
    info!("Initializing scrapers...");
    let leroy_scraper = Arc::new(LeroyScraper::new(
        Box::new((*zyte_adapter).clone())
    ).expect("Failed to create Leroy scraper"));

    let bauhaus_scraper = Arc::new(BauhausScraper::new(
        Box::new((*zyte_adapter).clone())
    ));

    let bricodepot_scraper = Arc::new(BricodepotScraper::new(
        Box::new((*zyte_adapter).clone())
    ));

    let scrapers = vec![
        leroy_scraper as Arc<dyn ScraperPort>,
        bauhaus_scraper as Arc<dyn ScraperPort>,
        bricodepot_scraper as Arc<dyn ScraperPort>,
    ];
    info!("Scrapers initialized successfully");

    // Initialize event publisher and handlers
    info!("Setting up event system...");
    let event_publisher: Arc<dyn EventPublisherPort> = Arc::new(InMemoryEventPublisher::new());
    let cache_handler = ProductCacheHandler::new(
        cache_adapter.clone(),
        event_publisher.clone(),
    );

    event_publisher.register_handler(Box::new(cache_handler)).await?;
    info!("Event system initialized");

    // Initialize scraper service
    let scraper_service = Arc::new(ScraperService::new(scrapers));
    info!("Scraper service initialized");

    // Create channel for worker communication
    let (tx, mut rx) = mpsc::channel::<ScrapeTask>(100);
    info!("Communication channel created");

    // Start consumers for each store
    info!("Starting store-specific consumers...");
    for store in &[Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
        let store_queue = queue_adapter.clone();
        let store_tx = tx.clone();
        let store_clone = store.clone();
        
        info!("Starting consumer for store: {}", store);
        tokio::spawn(async move {
            if let Err(e) = store_queue.consume_messages(&store_clone, store_tx).await {
                error!("Error in consumer for store {}: {}", store_clone, e);
            }
        });
    }

    // Start worker
    info!("Worker ready to process tasks");
    loop {
        tokio::select! {
            Some(task) = rx.recv() => {
                info!("Received task - Store: {}, Query: {}", task.store, task.query);
                match scraper_service.scrape_store_products(task.store, &task.query, task.num_products).await {
                    Ok(products) => {
                        info!("Successfully scraped {} products for store {} with query: {}", products.len(), task.store, task.query);
                        
                        // Cache the products for this store
                        if let Err(e) = cache_adapter.cache_store_products(&task.query, &task.store, &products).await {
                            error!("Failed to cache products for store {}: {}", task.store, e);
                        } else {
                            info!("Successfully cached {} products for store {} with query: {}", products.len(), task.store, task.query);
                        }

                        // Emit ProductsScraped event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::ProductsScraped {
                                metadata: EventMetadata::new(),
                                query: task.query.clone(),
                                products: products.clone(),
                                store: task.store.clone(),
                            })
                            .await
                        {
                            error!("Failed to publish ProductsScraped event: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to scrape products for store {} with query {}: {}", task.store, task.query, e);
                    }
                }
            }
        }
    }

    info!("Worker service shutting down");
    Ok(())
} 