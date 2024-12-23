use actix_web::{web, HttpResponse, Scope};
use serde::Serialize;
use crate::api::response::ApiResponse;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(HealthResponse {
        status: "ok".to_string(),
    }))
}

pub fn health_routes() -> Scope {
    web::scope("/health")
        .route("", web::get().to(health_check))
} 