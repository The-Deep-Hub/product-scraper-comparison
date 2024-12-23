use actix_web::{web, HttpResponse};
use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    api::middleware::validation::{PasswordReset, PasswordUpdate},
    db::repositories::user::UserRepository,
};

pub async fn request_reset(
    data: web::Json<PasswordReset>,
    user_repo: web::Data<UserRepository>,
    redis: web::Data<redis::aio::ConnectionManager>,
) -> Result<HttpResponse, AppError> {
    // Validate request data
    data.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Check if user exists
    let user = user_repo
        .find_by_email(&data.email)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Generate reset token
    let reset_token = Uuid::new_v4().to_string();
    let key = format!("password_reset:{}", reset_token);

    // Store token in Redis with expiry (1 hour)
    let mut conn = redis.get_ref().clone();
    let _: () = redis::cmd("SET")
        .arg(&key)
        .arg(user.id.unwrap().to_hex())
        .arg("EX")
        .arg(3600)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::RedisError(e))?;

    // TODO: Send email with reset token
    // For now, just return the token in the response (for testing)
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password reset token generated",
        "token": reset_token
    })))
}

pub async fn confirm_reset(
    data: web::Json<PasswordUpdate>,
    user_repo: web::Data<UserRepository>,
    redis: web::Data<redis::aio::ConnectionManager>,
) -> Result<HttpResponse, AppError> {
    // Validate request data
    data.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Get user ID from Redis
    let mut conn = redis.get_ref().clone();
    let key = format!("password_reset:{}", data.reset_token);
    let user_id: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::InternalServerError(e.into()))?;

    let user_id = user_id.ok_or_else(|| AppError::BadRequest("Invalid or expired token".to_string()))?;

    // Delete token from Redis
    let _: () = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::RedisError(e))?;

    // Hash new password
    let password_hash = hash(data.password.as_bytes(), DEFAULT_COST)
        .map_err(|e| AppError::InternalServerError(e.into()))?;

    // Update user password
    let user_id = bson::oid::ObjectId::parse_str(&user_id)
        .map_err(|e| AppError::InternalServerError(e.into()))?;

    let update = bson::doc! {
        "$set": {
            "password": password_hash,
            "updated_at": chrono::Utc::now()
        }
    };

    user_repo.update_one(user_id, update).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password updated successfully"
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/password")
            .route("/reset", web::post().to(request_reset))
            .route("/reset/confirm", web::post().to(confirm_reset))
    );
} 