use rust_scraper::{
    adapters::{
        outbound::{
            cache::RedisAdapter,
            queue::RabbitMQAdapter,
            http::ZyteAdapter,
            scrapers::{LeroyScraper, BauhausScraper, BricodepotScraper},
            events::{InMemoryEventPublisher, handlers::{ProductCacheHandler, MetricsHandler}},
        },
        inbound::worker::TaskProcessor,
    },
    domain::{
        ports::outbound::{ScraperPort, CachePort, EventPublisherPort},
    },
    config::builder,
    error::{AppResult, AppError},
};
use tracing::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,worker=debug")
        .init();

    // Load configuration
    let config = builder::new().expect("Failed to load configuration");

    // Initialize adapters
    let redis_adapter: Arc<dyn CachePort> = Arc::new(RedisAdapter::new().await.expect("Failed to create Redis adapter"));
    let rabbitmq_adapter = Arc::new(RabbitMQAdapter::new(
        config.amqp_url(),
        config.redis_url(),
    ).await.expect("Failed to create RabbitMQ adapter"));
    let zyte_adapter = Arc::new(ZyteAdapter::new(config.zyte.api_key.clone()));

    // Initialize scrapers
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

    // Create event publisher
    let event_publisher: Arc<dyn EventPublisherPort> = Arc::new(InMemoryEventPublisher::new());

    // Create handlers
    let cache_handler = ProductCacheHandler::new(
        Arc::clone(&redis_adapter),
        Arc::clone(&event_publisher)
    );
    let metrics_handler = MetricsHandler::new();

    // Register handlers
    event_publisher.register_handler(Box::new(cache_handler)).await?;
    event_publisher.register_handler(Box::new(metrics_handler)).await?;

    // Create task processor with correct arguments
    let processor = TaskProcessor::new(
        rabbitmq_adapter,            // queue_port
        Arc::new(scrapers),          // scrapers
        event_publisher,             // event_publisher
    );

    info!("Starting worker for store-specific queues");
    processor.run().await.map_err(AppError::from)?;

    Ok(())
} 