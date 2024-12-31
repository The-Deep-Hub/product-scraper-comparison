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
};
use std::sync::Arc;
use tracing::info;

const STORE_QUEUES: &[&str] = &[
    "leroy_tasks",
    "bauhaus_tasks",
    "bricodepot_tasks",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,worker=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Get configuration from environment
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let amqp_url = std::env::var("AMQP_ADDR").expect("AMQP_ADDR must be set");
    let zyte_api_key = std::env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set");

    // Initialize adapters
    let redis_adapter = Arc::new(RedisAdapter::new().await?);
    let rabbitmq_adapter = Arc::new(RabbitMQAdapter::new(amqp_url.clone(), redis_url.clone()).await?);
    let zyte_adapter = Arc::new(ZyteAdapter::new(zyte_api_key));

    // Initialize scrapers with individual http clients
    let leroy_scraper = Arc::new(LeroyScraper::new(Box::new((*zyte_adapter).clone()))?);
    let bauhaus_scraper = Arc::new(BauhausScraper::new(Box::new((*zyte_adapter).clone())));
    let bricodepot_scraper = Arc::new(BricodepotScraper::new(Box::new((*zyte_adapter).clone())));

    let scrapers = vec![
        leroy_scraper as Arc<dyn ScraperPort>,
        bauhaus_scraper as Arc<dyn ScraperPort>,
        bricodepot_scraper as Arc<dyn ScraperPort>,
    ];

    // Initialize scraper service
    let scraper_service = Arc::new(ScraperService::new(scrapers));

    info!("Starting unified worker for queues: {:?}", STORE_QUEUES);

    // Create and run task processor with all store queues
    let processor = TaskProcessor::new(
        amqp_url,
        STORE_QUEUES.iter().map(|&q| q.to_string()).collect(),
        redis_adapter,
        rabbitmq_adapter,
        scraper_service,
    );

    processor.run().await?;

    Ok(())
} 