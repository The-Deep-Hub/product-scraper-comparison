use actix_web::{HttpResponse, ResponseError};
use bcrypt::BcryptError;
use csv;
use serde_json::json;
use validator::ValidationErrors;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Validation Error: {0}")]
    ValidationError(#[from] ValidationErrors),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("User Already Exists")]
    UserAlreadyExists,

    #[error("Task Not Found: {0}")]
    TaskNotFound(String),

    #[error("MongoDB Error: {0}")]
    MongoError(#[from] mongodb::error::Error),

    #[error("Redis Error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("RabbitMQ Error: {0}")]
    RabbitMQError(#[from] lapin::Error),

    #[error("HTTP Client Error: {0}")]
    HttpClientError(#[from] reqwest::Error),

    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV Error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("CSV Writer Error: {0}")]
    CsvWriterError(#[from] csv::IntoInnerError<csv::Writer<Vec<u8>>>),

    #[error("UTF8 Error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Bcrypt Error: {0}")]
    BcryptError(#[from] BcryptError),

    #[error("BSON Error: {0}")]
    BsonError(#[from] bson::oid::Error),

    #[error("JWT Error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Bad Request",
                    "message": msg,
                }))
            }
            AppError::ValidationError(errors) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Validation Error",
                    "message": format!("{}", errors),
                }))
            }
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not Found",
                    "message": msg,
                }))
            }
            AppError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Unauthorized",
                    "message": msg,
                }))
            }
            AppError::UserAlreadyExists => {
                HttpResponse::BadRequest().json(json!({
                    "error": "User Already Exists",
                    "message": "A user with this email already exists",
                }))
            }
            AppError::TaskNotFound(task_id) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Task Not Found",
                    "message": format!("Task with id {} not found", task_id),
                }))
            }
            _ => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal Server Error",
                    "message": self.to_string(),
                }))
            }
        }
    }
} 