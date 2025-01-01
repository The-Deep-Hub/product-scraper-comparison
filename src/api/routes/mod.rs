pub mod health;
pub mod scraper;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/scraper")
            .service(scraper::search_products)
            .service(scraper::get_task_status)
            .service(scraper::get_product_details)
    )
    .service(
        web::scope("/health")
            .service(health::health_check)
    );
}