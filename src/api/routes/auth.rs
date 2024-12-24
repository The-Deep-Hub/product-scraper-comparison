use actix_web::{
    web::{self, Data, Json},
    HttpResponse,
};
use bcrypt;

use crate::{
    db::{models::user::User, repositories::user::UserRepository},
    error::{AppError, AppResult},
    models::auth::{Claims, LoginRequest, RegisterRequest},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    );
}

async fn register(
    Json(request): Json<RegisterRequest>,
    repo: Data<UserRepository>,
) -> AppResult<HttpResponse> {
    if request.password != request.password_confirmation {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Password Mismatch",
            "message": "Password and confirmation do not match"
        })));
    }

    let user = User::new(request.email.clone(), request.password);
    repo.get_ref().create_user(&user).await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "User created successfully",
        "email": request.email
    })))
}

async fn login(
    repo: Data<UserRepository>,
    Json(request): Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    let user = repo.get_ref().find_by_email(&request.email).await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    let is_valid = bcrypt::verify(&request.password, &user.password)?;

    if !is_valid {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid Credentials",
            "message": "Email or password is incorrect"
        })));
    }

    let expiration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 24 * 3600; // 24 hours from now

    let claims = Claims {
        sub: user.id.unwrap().to_string(),
        email: user.email,
        role: user.role,
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Login successful",
        "token": token,
        "email": claims.email,
        "role": claims.role
    })))
} 