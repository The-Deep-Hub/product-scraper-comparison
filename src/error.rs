use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use validator::ValidationErrors;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("MongoDB error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    
    #[error("RabbitMQ error: {0}")]
    RabbitMQError(#[from] lapin::Error),
    
    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };

        HttpResponse::build(status_code).json(response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::UserAlreadyExists => StatusCode::CONFLICT,
            AppError::TaskNotFound(_) => StatusCode::NOT_FOUND,
            AppError::MongoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RabbitMQError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
} 