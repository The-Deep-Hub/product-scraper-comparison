use actix_web::{web, App, HttpServer};
use tracing::{info, debug};
use std::sync::Arc;

use rust_scraper::{
    api::routes,
    core::app_state::AppState,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize application state
    let app_state = AppState::new()
        .await
        .expect("Failed to initialize application state");

    let server_config = &app_state.config.server;
    debug!("Server configuration: {:?}", server_config);
    info!("Starting HTTP server on {}:{}", server_config.host, server_config.port);

    let bind_addr = (server_config.host.as_str(), server_config.port);
    debug!("Binding to address: {:?}", bind_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&app_state.cache_service)))
            .app_data(web::Data::new(Arc::clone(&app_state.queue_service)))
            .app_data(web::Data::new(Arc::clone(&app_state.scraper_service)))
            .app_data(web::Data::new(Arc::clone(&app_state.task_splitter)))
            .configure(routes::configure)
    })
    .bind(bind_addr)?
    .run()
    .await
}
