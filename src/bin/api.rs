use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use lapin::Connection;
use rust_scraper::{
    api::routes::scraper::{search_products, get_task_status},
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        task_splitter::{TaskSplitterService, RabbitMQTaskSplitter},
    },
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug")
        .with_thread_ids(true)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize Redis connection
    let redis_url = format!(
        "redis://{}:{}@{}:{}/",
        std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
        std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
        std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
    );

    // Initialize RabbitMQ connection
    let rabbitmq_url = format!(
        "amqp://{}:{}@{}:{}",
        std::env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string()),
        std::env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string()),
        std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string()),
    );

    // Initialize services
    let cache_service: Arc<dyn CacheService> = Arc::new(RedisCacheService::new(&redis_url).await.unwrap());
    let queue_service: Arc<dyn QueueService> = Arc::new(RabbitMQQueue::new().await.unwrap());
    
    // Create RabbitMQ channel for task splitter
    let conn = Connection::connect(&rabbitmq_url, Default::default()).await.unwrap();
    let channel = conn.create_channel().await.unwrap();
    let task_splitter: Arc<dyn TaskSplitterService> = Arc::new(RabbitMQTaskSplitter::new(channel, cache_service.clone()));

    println!("Starting API server at http://127.0.0.1:8080");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cache_service.clone()))
            .app_data(web::Data::new(queue_service.clone()))
            .app_data(web::Data::new(task_splitter.clone()))
            .service(search_products)
            .service(get_task_status)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
