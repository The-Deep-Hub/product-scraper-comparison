use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

use rust_scraper::{
    adapters::{
        outbound::{
            cache::RedisAdapter,
            queue::RabbitMQAdapter,
            events::{
                InMemoryEventPublisher,
                handlers::ProductCacheHandler,
            },
        },
        inbound::api::routes::ApiRoutes,
    },
    domain::{
        ports::outbound::{CachePort, QueuePort, EventPublisherPort},
        config::QueueConfig,
    },
    config::builder,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with better configuration
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("rust_scraper=debug,api=debug"))
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

    // Initialize event publisher and handlers
    info!("Setting up event system...");
    let event_publisher = Arc::new(InMemoryEventPublisher::new());
    let cache_handler = Box::new(ProductCacheHandler::new(
        cache_adapter.clone(),
        event_publisher.clone(),
    ));

    event_publisher.register_handler(cache_handler).await
        .expect("Failed to register cache handler");
    info!("Event system initialized");

    // Create server
    info!("Starting HTTP server on {}:{}", config.server.host, config.server.port);
    HttpServer::new(move || {
        // Create CORS middleware inside the closure
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .configure(ApiRoutes::configure(
                cache_adapter.clone() as Arc<dyn CachePort>,
                queue_adapter.clone() as Arc<dyn QueuePort>,
            ))
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
