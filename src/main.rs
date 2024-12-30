use actix_web::{web, App, HttpServer};
use tracing::info;
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

    info!("Starting HTTP server");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&app_state.cache_service)))
            .app_data(web::Data::new(Arc::clone(&app_state.queue_service)))
            .app_data(web::Data::new(Arc::clone(&app_state.scraper_service)))
            .app_data(web::Data::new(Arc::clone(&app_state.task_splitter)))
            .configure(routes::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
