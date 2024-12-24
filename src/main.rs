use std::sync::Arc;
use std::time::Duration;

use api::start_server;
use clients::zyte::ZyteClient;
use services::cache::RedisCache;
use services::queue::RabbitMQQueue;
use services::scraper::ScraperServiceImpl;
use services::worker::WorkerService;
use tracing::info;

mod api;
mod clients;
mod db;
mod error;
mod middleware;
mod models;
mod scrapers;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting application");

    // Initialize services
    let zyte_client = ZyteClient::new()?;
    let cache_service = Arc::new(RedisCache::new().await?);
    let queue_service = Arc::new(RabbitMQQueue::new().await?);
    let scraper_service = Arc::new(ScraperServiceImpl::new(
        zyte_client.clone(),
        Arc::clone(&cache_service),
        Arc::clone(&queue_service),
    ));

    // Initialize worker service
    let worker_service = WorkerService::new(
        Arc::clone(&queue_service),
        Arc::clone(&scraper_service),
        Duration::from_secs(5),
        5,
    );

    // Start worker service in a separate task
    tokio::spawn(async move {
        if let Err(e) = worker_service.start().await {
            eprintln!("Worker service error: {}", e);
        }
    });

    // Start API server
    start_server(
        "127.0.0.1:8080",
        Arc::clone(&cache_service),
        Arc::clone(&queue_service),
        Arc::clone(&scraper_service),
    )
    .await?;

    Ok(())
}
