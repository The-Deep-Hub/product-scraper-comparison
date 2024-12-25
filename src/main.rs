use std::sync::Arc;
use actix_web::{web, App, HttpServer, middleware};
use dotenv::dotenv;
use tracing::info;

use crate::{
    api::{middleware::Logger, routes},
    clients::zyte::ZyteClient,
    scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper},
    services::{
        cache::{RedisCacheService, CacheService},
        queue::{RabbitMQQueue, QueueService},
        scraper::{CombinedScraperService, ScraperService},
        worker::WorkerService,
    },
};

mod api;
mod clients;
mod error;
mod models;
mod scrapers;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize services
    let zyte_client = ZyteClient::new().expect("Failed to create Zyte client");
    
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let cache_service = Arc::new(RedisCacheService::new(&redis_url).await.expect("Failed to create Redis cache service"));
    let queue_service = Arc::new(RabbitMQQueue::new().await.expect("Failed to create RabbitMQ queue service"));
    
    // Initialize scrapers
    let leroy_scraper = LeroyScraper::new(zyte_client.clone());
    let bauhaus_scraper = BauhausScraper::new(zyte_client.clone());
    let bricodepot_scraper = BricodepotScraper::new(zyte_client);
    
    // Create combined scraper service
    let scraper_service = Arc::new(CombinedScraperService::new(
        leroy_scraper,
        bauhaus_scraper,
        bricodepot_scraper,
        Box::new(RedisCacheService::new(&redis_url).await.expect("Failed to create Redis cache service")),
        Box::new(RabbitMQQueue::new().await.expect("Failed to create RabbitMQ queue service")),
    ));
    
    // Create worker service
    let worker_service = WorkerService::new(
        Arc::clone(&queue_service) as Arc<dyn QueueService>,
        scraper_service.clone() as Arc<dyn ScraperService>,
    );
    
    // Start worker service in background
    tokio::spawn(async move {
        if let Err(e) = worker_service.run().await {
            tracing::error!("Worker service error: {}", e);
        }
    });
    
    info!("Starting HTTP server...");
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger)
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .app_data(web::Data::new(scraper_service.clone() as Arc<dyn ScraperService>))
            .app_data(web::Data::new(cache_service.clone() as Arc<dyn CacheService>))
            .app_data(web::Data::new(queue_service.clone() as Arc<dyn QueueService>))
            .service(
                web::scope("/api")
                    .service(routes::health::healthcheck)
                    .service(
                        web::scope("/scraper")
                            .service(routes::scraper::search_products)
                            .service(routes::scraper::get_product_details)
                            .service(routes::scraper::get_task_status)
                    )
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
