pub mod api;
pub mod config;
pub mod core;
pub mod db;
pub mod error;
pub mod logging;
pub mod middleware;
pub mod workers;
pub mod domain;
pub mod adapters;

pub use error::{AppError, AppResult};

use adapters::inbound::api::ApiRoutes;
use domain::{
    services::SearchService,
    ports::outbound::{CachePort, QueuePort, HttpClientPort},
};

pub fn configure_api<C, Q, H>(search_service: SearchService<C, Q, H>) -> impl FnOnce(&mut actix_web::web::ServiceConfig)
where
    C: CachePort + Clone + 'static,
    Q: QueuePort + Clone + 'static,
    H: HttpClientPort + Clone + 'static,
{
    ApiRoutes::configure(search_service)
}
