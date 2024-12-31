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
        services::scraper::ScraperService,
        ports::outbound::{HttpClientPort, ScraperPort},
    },
    workers::processor::TaskProcessor,
    config::builder::AppConfig,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,worker=debug")
        .init();

    // Load configuration
    let config = AppConfig::new()?;

    // Initialize adapters
    let redis_adapter = Arc::new(RedisAdapter::new().await?);
    let rabbitmq_adapter = Arc::new(RabbitMQAdapter::new(
        config.amqp_url(),
        config.redis_url(),
    ).await?);
    let zyte_adapter = Arc::new(ZyteAdapter::new(config.zyte.api_key.clone()));

    // Initialize scrapers
    let leroy_scraper = Arc::new(LeroyScraper::new(
        Box::new((*zyte_adapter).clone())
    )?);
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

    // Initialize scraper service
    let scraper_service = Arc::new(ScraperService::new(scrapers));

    info!("Starting unified worker for queues: {:?}", config.worker.get_store_queues());

    // Create and run task processor with all store queues
    let processor = TaskProcessor::new(
        config.amqp_url(),
        config.worker.get_store_queues(),
        redis_adapter,
        rabbitmq_adapter,
        scraper_service,
    );

    processor.run().await?;

    Ok(())
} 