use rust_scraper::api::{config::ApiConfig, middleware::logging::setup_logging, ApiServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    setup_logging();

    // Load configuration
    let config = ApiConfig::new();

    // Create and run API server
    let server = ApiServer::new(config);
    server.run().await
}
