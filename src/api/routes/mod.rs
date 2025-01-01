pub mod health;
pub mod scraper;

use actix_web::web;
use crate::{
    api::AppState,
    domain::ports::outbound::{CachePort, QueuePort},
};

pub fn configure<C, Q>(cfg: &mut web::ServiceConfig)
where
    C: CachePort + 'static,
    Q: QueuePort + 'static,
{
    cfg.service(
        web::scope("/scraper")
            .route("/search", web::post().to(scraper::search_products::<C, Q>))
            .route("/task/{task_id}", web::get().to(scraper::get_task_status::<C, Q>))
            .route("/product/{url}", web::get().to(scraper::get_product_details::<C, Q>))
    )
    .service(
        web::scope("/health")
            .service(health::health_check)
    );
}