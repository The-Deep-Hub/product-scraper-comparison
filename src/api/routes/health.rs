use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};
use chrono::Utc;

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(health_check);
} 