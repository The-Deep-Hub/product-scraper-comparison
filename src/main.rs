use rust_scraper::api::{
    config::ApiConfig,
    middleware::auth::{AuthMiddleware, AuthConfig},
};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = ApiConfig::default();
    let auth_config = AuthConfig::default();
    let auth_middleware = AuthMiddleware::new(auth_config);

    let listener = TcpListener::bind((config.host.as_str(), config.port))?;
    println!("Server running at http://{}:{}", config.host, config.port);

    rust_scraper::api::start_server(listener, auth_middleware).await
}
