use actix_web::web;
use crate::services::queue::QueueService;

pub mod routes;

pub fn configure_app<Q: QueueService + 'static>(
    cfg: &mut web::ServiceConfig,
    queue: Q,
) {
    routes::config(cfg, queue);
}
