use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use validator::ValidationErrors;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    InternalServerError(anyhow::Error),
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    UserAlreadyExists,
    TaskNotFound(String),
    DatabaseError(mongodb::error::Error),
    RedisError(redis::RedisError),
    RabbitMQError(lapin::Error),
    ValidationError(ValidationErrors),
    JWTError(jsonwebtoken::errors::Error),
    CSVError(csv::Error),
    UTF8Error(std::string::FromUtf8Error),
    CSVWriterError(csv::IntoInnerError<csv::Writer<Vec<u8>>>),
    SerdeError(serde_json::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InternalServerError(err) => write!(f, "Internal Server Error: {}", err),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::UserAlreadyExists => write!(f, "User already exists"),
            AppError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            AppError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AppError::RedisError(err) => write!(f, "Redis error: {}", err),
            AppError::RabbitMQError(err) => write!(f, "RabbitMQ error: {}", err),
            AppError::ValidationError(err) => write!(f, "Validation error: {}", err),
            AppError::JWTError(err) => write!(f, "JWT error: {}", err),
            AppError::CSVError(err) => write!(f, "CSV error: {}", err),
            AppError::UTF8Error(err) => write!(f, "UTF-8 error: {}", err),
            AppError::CSVWriterError(err) => write!(f, "CSV writer error: {}", err),
            AppError::SerdeError(err) => write!(f, "Serde error: {}", err),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InternalServerError(ref err) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal Server Error",
                    "message": err.to_string(),
                }))
            }
            AppError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Bad Request",
                    "message": message,
                }))
            }
            AppError::Unauthorized(ref message) => {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Unauthorized",
                    "message": message,
                }))
            }
            AppError::NotFound(ref message) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not Found",
                    "message": message,
                }))
            }
            AppError::UserAlreadyExists => {
                HttpResponse::BadRequest().json(json!({
                    "error": "User Already Exists",
                    "message": "A user with this email already exists",
                }))
            }
            AppError::TaskNotFound(ref task_id) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Task Not Found",
                    "message": format!("Task with id {} not found", task_id),
                }))
            }
            AppError::SerdeError(ref err) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Invalid JSON",
                    "message": err.to_string(),
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

impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::RedisError(err)
    }
}

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::ValidationError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalServerError(err)
    }
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> Self {
        AppError::CSVError(err)
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AppError::UTF8Error(err)
    }
}

impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for AppError {
    fn from(err: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Self {
        AppError::CSVWriterError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerdeError(err)
    }
} 