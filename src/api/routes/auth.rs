use actix_web::{
    web::{self, Data, Json, ServiceConfig},
    HttpResponse,
    Responder,
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;

use crate::{
    api::{
        middleware::auth::Role,
        models::auth::{LoginRequest, RegisterRequest},
        utils::generate_token,
    },
    db::{
        repositories::{user::UserRepository, Repository},
        models::user::User,
    },
};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    );
}

async fn register(
    repo: Data<UserRepository>,
    request: Json<RegisterRequest>,
) -> impl Responder {
    // Check if user already exists
    match repo.exists_by_email(&request.email).await {
        Ok(true) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "User with this email already exists"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to check user existence"
            }));
        }
        _ => {}
    }

    // Hash password
    let password_hash = match hash(request.password.as_bytes(), DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to hash password"
            }));
        }
    };

    // Create user
    let now = Utc::now();
    let user = User {
        id: None,
        email: request.email.clone(),
        password_hash,
        role: Role::User,
        created_at: now,
        updated_at: now,
    };

    // Save user to database
    match repo.create(user).await {
        Ok(created_user) => {
            HttpResponse::Ok().json(serde_json::json!({
                "message": "User registered successfully",
                "email": created_user.email,
                "role": created_user.role.to_string()
            }))
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create user"
            }))
        }
    }
}

async fn login(
    repo: Data<UserRepository>,
    request: Json<LoginRequest>,
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
    if !bcrypt::verify(&request.password, &user.password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid email or password"
        }));
    }

    // Generate JWT token
    match generate_token(&user.email, &user.role) {
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Login successful",
            "token": token,
            "email": user.email,
            "role": user.role.to_string()
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to generate token"
        }))
    }
} 