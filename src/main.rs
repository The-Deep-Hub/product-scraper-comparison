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

    let server_addr = app_state.config.server_addr();
    info!("Starting HTTP server at {}", server_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(routes::configure)
    })
    .bind(server_addr)?
    .run()
    .await
}
