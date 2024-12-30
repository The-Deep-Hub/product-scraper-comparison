use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[get("")]
pub async fn health_check() -> impl Responder {
    let response = HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    };
    HttpResponse::Ok().json(response)
}