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
            scrapers::{LeroyScraper, BauhausScraper, BricodepotScraper},
        },
    },
    domain::{
        services::scraper::ScraperService,
        ports::outbound::ScraperPort,
    },
    config::builder::AppConfig,
    api::{self, AppState},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,api=debug")
        .init();

    // Load configuration
    let config = AppConfig::new().expect("Failed to load configuration");

    // Initialize adapters
    let redis_adapter = Arc::new(RedisAdapter::new().await.expect("Failed to create Redis adapter"));
    let rabbitmq_adapter = Arc::new(RabbitMQAdapter::new(
        config.amqp_url(),
        config.redis_url(),
    ).await.expect("Failed to create RabbitMQ adapter"));
    let zyte_adapter = Arc::new(ZyteAdapter::new(config.zyte.api_key.clone()));

    // Initialize scrapers
    let leroy_scraper = Arc::new(LeroyScraper::new(
        Box::new((*zyte_adapter).clone())
    ).expect("Failed to create Leroy scraper"));
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
    
    // Create app state
    let app_state = web::Data::new(AppState::<RedisAdapter, RabbitMQAdapter>::new(
        scraper_service,
        redis_adapter.clone(),
        rabbitmq_adapter.clone(),
    ));
    
    info!("Starting HTTP server at {}", config.server_addr());

    HttpServer::new(move || {
        // Create new CORS middleware for each worker
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .configure(api::configure::<RedisAdapter, RabbitMQAdapter>)
    })
    .bind(config.server_addr())?
    .run()
    .await
}
