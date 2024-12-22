pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod response;
pub mod routes;

use actix_web::{middleware::Logger, App, HttpServer};
use tracing::info;

use crate::api::{
    config::ApiConfig,
    middleware::cors_middleware,
    routes::health_config,
};

pub struct ApiServer {
    config: ApiConfig,
}

impl ApiServer {
    pub fn new(config: ApiConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        info!("Starting API server at http://{}", addr);

        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .wrap(cors_middleware())
                .configure(health_config)
        })
        .bind(&addr)?
        .run()
        .await
    }
}
