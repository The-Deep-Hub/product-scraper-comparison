use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use tracing::info;
use std::sync::Arc;

use rust_scraper::{
    adapters::{
        outbound::{
            cache::RedisAdapter,
            queue::RabbitMQAdapter,
            http::ZyteAdapter,
        },
        inbound::api::{ApiRoutes, Logger},
    },
    domain::services::SearchService,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize adapters
    let redis_adapter = RedisAdapter::new()
        .await
        .expect("Failed to initialize Redis adapter");
        
    let rabbitmq_adapter = RabbitMQAdapter::new(
        std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set"),
        std::env::var("REDIS_URL").expect("REDIS_URL must be set"),
    )
    .await
    .expect("Failed to initialize RabbitMQ adapter");
    
    let zyte_adapter = ZyteAdapter::new(
        std::env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set"),
    );

    // Initialize search service
    let search_service = SearchService::new(
        redis_adapter,
        rabbitmq_adapter,
        zyte_adapter,
    );

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_addr = format!("{}:{}", host, port);
    
    info!("Starting HTTP server at {}", server_addr);

    HttpServer::new(move || {
        // Create new CORS middleware for each worker
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger)
            .configure(ApiRoutes::configure(search_service.clone()))
    })
    .bind(server_addr)?
    .run()
    .await
}
