use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use tracing::info;

use rust_scraper::{
    api::routes,
    clients::zyte::ZyteClient,
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        scraper::{ScraperService, CombinedScraperService},
        task_splitter::{TaskSplitterService, RabbitMQTaskSplitter},
    },
    scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper},
};

struct AppServices {
    cache_service: Arc<dyn CacheService>,
    queue_service: Arc<dyn QueueService>,
    scraper_service: Arc<dyn ScraperService>,
    task_splitter: Arc<dyn TaskSplitterService>,
}

async fn init_services() -> Result<AppServices, Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize Redis
    let redis_url = format!(
        "redis://{}:{}@{}:{}/",
        std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
        std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
        std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
    );
    let redis_password = std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set");
    info!("Connecting to Redis at: {}", redis_url.replace(&redis_password, "****"));
    
    // Initialize services
    let cache_service = Arc::new(RedisCacheService::new(&redis_url).await?) as Arc<dyn CacheService>;
    
    // Initialize RabbitMQ connection
    let amqp_url = std::env::var("AMQP_ADDR").expect("AMQP_ADDR must be set");
    let conn = lapin::Connection::connect(
        &amqp_url,
        lapin::ConnectionProperties::default(),
    ).await?;
    let channel = conn.create_channel().await?;
    
    // Initialize queue service
    let queue_service = Arc::new(RabbitMQQueue::new().await?) as Arc<dyn QueueService>;
    
    // Initialize task splitter
    let task_splitter = Arc::new(RabbitMQTaskSplitter::new(
        channel,
        Arc::clone(&cache_service),
    )) as Arc<dyn TaskSplitterService>;
    
    task_splitter.setup_queues().await?;
    
    // Initialize Zyte client
    let zyte_client = ZyteClient::new()?;
    
    // Initialize individual scrapers
    let leroy_scraper = LeroyScraper::new(zyte_client.clone());
    let bauhaus_scraper = BauhausScraper::new(zyte_client.clone());
    let bricodepot_scraper = BricodepotScraper::new(zyte_client);
    
    // Create combined scraper service
    let scraper_service = Arc::new(CombinedScraperService::new(
        leroy_scraper,
        bauhaus_scraper,
        bricodepot_scraper,
        Box::new(RedisCacheService::new(&redis_url).await?),
        Box::new(RabbitMQQueue::new().await?),
    )) as Arc<dyn ScraperService>;

    Ok(AppServices {
        cache_service,
        queue_service,
        scraper_service,
        task_splitter,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug")
        .init();

    // Initialize services
    let services = init_services()
        .await
        .expect("Failed to initialize services");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&services.cache_service)))
            .app_data(web::Data::new(Arc::clone(&services.queue_service)))
            .app_data(web::Data::new(Arc::clone(&services.scraper_service)))
            .app_data(web::Data::new(Arc::clone(&services.task_splitter)))
            .service(routes::scraper::search_products)
            .service(routes::scraper::get_task_status)
            .service(routes::scraper::get_product_details)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
