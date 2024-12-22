use actix_web::{get, web, HttpResponse};
use serde::Serialize;

use crate::api::response::ApiResponse;

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    status: String,
    version: String,
}

pub fn health_config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}

#[get("/health")]
async fn health_check() -> HttpResponse {
    let health_status = HealthStatus {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    HttpResponse::Ok().json(ApiResponse::success(health_status))
} 