use actix_web::{web, HttpResponse, Responder};
use validator::Validate;
use std::str::FromStr;

use crate::{
    error::AppError,
    db::{
        models::user::User,
    },
    db::repositories::user::UserRepository,
    api::{
        middleware::auth::Role,
        models::auth::{LoginRequest, RegisterRequest},
        utils::generate_token,
    },
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    );
}

async fn register(
    data: web::Json<RegisterRequest>,
    user_repo: web::Data<UserRepository>,
) -> Result<HttpResponse, AppError> {
    data.validate()?;

    match user_repo.find_by_email(&data.email).await? {
        Some(_) => Err(AppError::UserAlreadyExists),
        None => {
            let user = User::new(data.email.clone(), data.password.clone());
            let _id = user_repo.create_user(&user).await?;
            Ok(HttpResponse::Created().json(user))
        }
    }
}

async fn login(
    repo: web::Data<UserRepository>,
    request: web::Json<LoginRequest>,
) -> impl Responder {
    // Find user by email
    let user = match repo.find_by_email(&request.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid email or password"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to find user"
            }));
        }
    };

    // Verify password
    if !bcrypt::verify(&request.password, &user.password).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid email or password"
        }));
    }

    // Generate JWT token
    let role = Role::from_str(&user.role).unwrap_or(Role::User);
    match generate_token(&user.email, &role) {
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Login successful",
            "token": token,
            "email": user.email,
            "role": user.role
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to generate token"
        }))
    }
} 