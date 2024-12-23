use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use tracing::{info, debug};
use crate::api::{
    middleware::auth::{AuthMiddleware, Role},
    models::auth::{LoginRequest, RegisterRequest, User, UserResponse, AuthResponse},
    error::ApiError,
};

/// Register a new user (requires admin authentication)
pub async fn register(
    req: web::Json<RegisterRequest>,
    auth_middleware: web::Data<AuthMiddleware>,
    auth: Option<BearerAuth>,
) -> Result<impl Responder, ApiError> {
    debug!("Received registration request for email: {}", req.email);

    // Extract token from header
    let token = auth
        .map(|auth| auth.token().to_string())
        .ok_or_else(|| {
            debug!("Missing authorization header");
            ApiError::Unauthorized("Missing authorization header".into())
        })?;

    // Verify admin token
    let claims = auth_middleware.verify_token(&token)
        .map_err(|_| {
            debug!("Invalid token");
            ApiError::Unauthorized("Invalid token".into())
        })?;
    
    // Only admins can create new users
    if claims.role != Role::Admin {
        debug!("Non-admin user attempted to register new user");
        return Err(ApiError::Unauthorized("Admin privileges required".into()));
    }

    // Create new user
    let new_user = User {
        email: req.email.clone(),
        password: req.password.clone(), // In production, this would be hashed
        name: req.name.clone(),
        role: req.role.unwrap_or(Role::Basic),
    };

    // Generate token for new user
    let token = auth_middleware.generate_token(&new_user.email, new_user.role)
        .map_err(|_| {
            debug!("Failed to generate token for new user");
            ApiError::Unauthorized("Failed to generate token".into())
        })?;

    info!("Successfully registered new user: {}", new_user.email);
    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse {
            email: new_user.email,
            name: new_user.name,
            role: new_user.role,
        },
    }))
}

/// Login with existing credentials
pub async fn login(
    req: web::Json<LoginRequest>,
    auth_middleware: web::Data<AuthMiddleware>,
) -> Result<impl Responder, ApiError> {
    debug!("Received login request for email: {}", req.email);

    // TODO: Verify credentials against MongoDB
    // For now, just generate a token for testing
    let user = User {
        email: req.email.clone(),
        password: req.password.clone(),
        name: "Test User".to_string(),
        role: Role::Basic,
    };

    // Generate token
    let role = user.role;
    let token = auth_middleware.generate_token(&user.email, role)
        .map_err(|_| {
            debug!("Failed to generate token for user login");
            ApiError::Unauthorized("Failed to generate token".into())
        })?;

    info!("Successfully logged in user: {}", user.email);
    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse {
            email: user.email,
            name: user.name,
            role,
        },
    }))
}

pub fn auth_routes() -> actix_web::Scope {
    debug!("Configuring auth routes");
    web::scope("/auth")
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
} 