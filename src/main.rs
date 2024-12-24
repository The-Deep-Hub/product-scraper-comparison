use std::sync::Arc;
use actix_web::{web, App, HttpServer, middleware};
use dotenv::dotenv;
use tracing::info;

use crate::{
    api::{middleware::Logger, routes},
    clients::zyte::ZyteClient,
    scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper},
    services::{
        cache::RedisCacheService,
        queue::RabbitMQQueue,
        scraper::CombinedScraperService,
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
    
    let cache_service = Arc::new(RedisCacheService::new().await.expect("Failed to create Redis cache service"));
    let queue_service = Arc::new(RabbitMQQueue::new().await.expect("Failed to create RabbitMQ queue service"));
    
    // Initialize scrapers
    let leroy_scraper = LeroyScraper::new(zyte_client.clone()).expect("Failed to create Leroy scraper");
    let bauhaus_scraper = BauhausScraper::new(zyte_client.clone()).expect("Failed to create Bauhaus scraper");
    let bricodepot_scraper = BricodepotScraper::new(zyte_client).expect("Failed to create Bricodepot scraper");
    
    // Create combined scraper service
    let scraper_service = Arc::new(CombinedScraperService::new(
        leroy_scraper,
        bauhaus_scraper,
        bricodepot_scraper,
        Box::new(RedisCacheService::new().await.expect("Failed to create Redis cache service")),
        Box::new(RabbitMQQueue::new().await.expect("Failed to create RabbitMQ queue service")),
    ));
    
    // Create worker service
    let worker_service = WorkerService::new(
        Arc::clone(&queue_service),
        Arc::clone(&scraper_service),
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
            .app_data(web::Data::new(Arc::clone(&scraper_service)))
            .app_data(web::Data::new(Arc::clone(&cache_service)))
            .app_data(web::Data::new(Arc::clone(&queue_service)))
            .service(routes::health::healthcheck)
            .service(routes::scraper::search_products)
            .service(routes::scraper::get_product_details)
            .service(routes::scraper::get_task_status)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
