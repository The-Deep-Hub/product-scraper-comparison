use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use redis::AsyncCommands;
use uuid::Uuid;
use validator::Validate;

use crate::{
    api::{
        error::ApiError,
        middleware::validation::{PasswordReset, PasswordUpdate},
    },
    db::repositories::user::UserRepository,
};

pub async fn request_reset(
    data: web::Json<PasswordReset>,
    user_repo: web::Data<UserRepository>,
    redis: web::Data<redis::aio::ConnectionManager>,
) -> Result<impl Responder, ApiError> {
    // Validate request data
    data.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Check if user exists
    let user = user_repo
        .find_by_email(&data.email)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    // Generate reset token
    let reset_token = Uuid::new_v4().to_string();
    let key = format!("password_reset:{}", reset_token);

    // Store token in Redis with expiry (1 hour)
    let mut redis = redis.clone();
    redis
        .set_ex(&key, user.id.unwrap().to_hex(), 3600)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

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
) -> Result<impl Responder, ApiError> {
    // Validate request data
    data.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Get user ID from Redis
    let mut redis = redis.clone();
    let key = format!("password_reset:{}", data.reset_token);
    let user_id: Option<String> = redis
        .get(&key)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let user_id = user_id.ok_or_else(|| ApiError::BadRequest("Invalid or expired token".to_string()))?;

    // Delete token from Redis
    redis
        .del(&key)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Hash new password
    let password_hash = hash(data.password.as_bytes(), DEFAULT_COST)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // Update user password
    let user_id = bson::oid::ObjectId::parse_str(&user_id)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let update = bson::doc! {
        "$set": {
            "password_hash": password_hash,
            "updated_at": chrono::Utc::now()
        }
    };

    user_repo.update_by_id(user_id, update).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password updated successfully"
    })))
}

pub fn password_routes() -> actix_web::Scope {
    web::scope("/password")
        .route("/reset", web::post().to(request_reset))
        .route("/reset/confirm", web::post().to(confirm_reset))
} 