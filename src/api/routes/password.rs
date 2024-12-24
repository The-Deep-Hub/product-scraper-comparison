use actix_web::{web, HttpResponse};
use redis::AsyncCommands;
use serde::Deserialize;

use crate::{
    error::{AppError, AppResult},
    db::repositories::user::UserRepository,
};

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ConfirmResetRequest {
    pub token: String,
    pub new_password: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/password")
            .route("/reset", web::post().to(request_reset))
            .route("/confirm", web::post().to(confirm_reset)),
    );
}

async fn request_reset(
    request: web::Json<ResetPasswordRequest>,
    _repo: web::Data<UserRepository>,
    redis: web::Data<redis::aio::ConnectionManager>,
) -> AppResult<HttpResponse> {
    let mut conn = redis.get_ref().clone();

    let key = format!("reset_token:{}", request.email);
    let token = uuid::Uuid::new_v4().to_string();

    let _: () = conn.set_ex(
        &key,
        &token,
        3600, // 1 hour expiry
    )
    .await
    .map_err(|e| AppError::RedisError(e))?;

    Ok(HttpResponse::Ok().finish())
}

async fn confirm_reset(
    request: web::Json<ConfirmResetRequest>,
    _repo: web::Data<UserRepository>,
    redis: web::Data<redis::aio::ConnectionManager>,
) -> AppResult<HttpResponse> {
    let mut conn = redis.get_ref().clone();

    let key = format!("reset_token:{}", request.token);
    let stored_token: Option<String> = conn.get(&key).await.map_err(|e| AppError::RedisError(e))?;

    match stored_token {
        Some(token) if token == request.token => {
            let _: () = conn.del(&key)
                .await
                .map_err(|e| AppError::RedisError(e))?;

            Ok(HttpResponse::Ok().finish())
        }
        _ => Ok(HttpResponse::BadRequest().finish()),
    }
} 