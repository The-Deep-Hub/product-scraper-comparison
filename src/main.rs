use actix_web::{middleware, App, HttpServer};
use rust_scraper::{
    services::queue::RabbitMQQueue,
    api::configure_app,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize services
    let queue_service = RabbitMQQueue::new().await.expect("Failed to initialize queue service");
    
    // Create HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(|cfg| configure_app(cfg, queue_service.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
