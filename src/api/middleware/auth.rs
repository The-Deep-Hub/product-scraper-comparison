use actix_web::{
    dev::ServiceRequest,
    Error,
    error::ErrorUnauthorized,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Role {
    Admin,
    Basic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: Role,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiration: i64, // in seconds
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key".to_string(),
            token_expiration: 24 * 60 * 60, // 24 hours
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddleware {
    config: AuthConfig,
}

impl AuthMiddleware {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    pub fn generate_token(&self, user_id: &str, role: Role) -> Result<String, Error> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.token_expiration);
        
        let claims = Claims {
            sub: user_id.to_string(),
            role,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| ErrorUnauthorized(e.to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, Error> {
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| ErrorUnauthorized(e.to_string()))?;

        Ok(decoded.claims)
    }

    pub fn verify_request(&self, req: &ServiceRequest) -> Result<Claims, Error> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .ok_or_else(|| ErrorUnauthorized("Missing authorization header"))?;

        let auth_str = auth_header.to_str()
            .map_err(|_| ErrorUnauthorized("Invalid authorization header"))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(ErrorUnauthorized("Invalid authorization header format"));
        }

        let token = &auth_str[7..];
        self.verify_token(token)
    }
}

pub fn require_role(role: Role, config: AuthConfig) -> impl Fn(ServiceRequest) -> Pin<Box<dyn Future<Output = Result<ServiceRequest, Error>>>> {
    let middleware = AuthMiddleware::new(config);
    
    move |req: ServiceRequest| {
        let middleware = middleware.clone();
        Box::pin(async move {
            let claims = middleware.verify_request(&req)?;
            if claims.role != role {
                return Err(ErrorUnauthorized("Insufficient permissions"));
            }
            Ok(req)
        })
    }
} 