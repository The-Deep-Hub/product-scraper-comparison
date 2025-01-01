use rust_scraper::{
    adapters::{
        outbound::{
            cache::RedisAdapter,
            queue::RabbitMQAdapter,
            http::ZyteAdapter,
            scrapers::{LeroyScraper, BauhausScraper, BricodepotScraper},
        },
    },
    domain::{
        ports::outbound::ScraperPort,
    },
    config::builder::AppConfig,
    TaskProcessor,
};
use tracing::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,worker=debug")
        .init();

    // Load configuration
    let config = AppConfig::new().expect("Failed to load configuration");

    // Initialize adapters
    let redis_adapter = Arc::new(RedisAdapter::new().await.expect("Failed to create Redis adapter"));
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

    // Create and run task processor
    let processor = TaskProcessor::new(
        config.amqp_url(),
        redis_adapter.clone(),
        rabbitmq_adapter.clone(),
        Arc::new(scrapers),
    );

    info!("Starting worker for store-specific queues");
    processor.run().await?;

    Ok(())
} 