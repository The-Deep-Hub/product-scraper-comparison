use actix_web::web;
use crate::services::queue::QueueService;

pub mod scraper;

pub fn config<Q: QueueService + 'static>(
    cfg: &mut web::ServiceConfig,
    queue: Q,
) {
    cfg.app_data(web::Data::new(queue))
        .service(
            web::scope("/api")
                .configure(scraper::config)
        );
} 