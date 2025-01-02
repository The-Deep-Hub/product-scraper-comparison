use actix_cors::Cors;
use actix_web::{App, HttpServer};
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, EnvFilter};

use rust_scraper::{
    adapters::{
        inbound::api::routes::ApiRoutes,
        outbound::{
            cache::RedisAdapter,
            queue::RabbitMQAdapter,
            http::ZyteAdapter,
        },
    },
    config::builder,
    domain::config::QueueConfig,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with better configuration
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("rust_scraper=debug,api=debug,actix_web=info"))
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .with_level(true)
        .init();

    info!("Starting API service...");

    // Load configuration
    let config = builder::new().expect("Failed to load configuration");
    info!("Configuration loaded successfully");

    // Initialize adapters
    info!("Initializing adapters...");
    let cache_adapter = RedisAdapter::new(Box::new(config.cache.clone()))
        .await
        .expect("Failed to create Redis adapter");
    info!("Redis adapter initialized");

    let queue_adapter = RabbitMQAdapter::new(
        config.queue.connection_url(),
        config.queue.queue_name(),
    )
    .await
    .expect("Failed to create RabbitMQ adapter");
    info!("RabbitMQ adapter initialized");

    let zyte_adapter = ZyteAdapter::new(
        config.http_client.api_key.clone()
            .expect("Zyte API key must be configured")
    );
    info!("Zyte adapter initialized");

    // Configure CORS
    info!("Configuring CORS...");
    let cors = Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header();
    info!("CORS configured");

    // Start HTTP server
    let addr = config.server_addr();
    info!("Starting HTTP server at {}", addr);
    
    HttpServer::new(move || {
        // Clone adapters for each worker
        let cache = cache_adapter.clone();
        let queue = queue_adapter.clone();

        // Create CORS middleware for each worker
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        info!("Configuring new worker instance");
        App::new()
            .wrap(cors)
            .configure(ApiRoutes::configure(cache, queue))
    })
    .bind(addr)?
    .run()
    .await?;

    info!("API service shutting down");
    Ok(())
}
