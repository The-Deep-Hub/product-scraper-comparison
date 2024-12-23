use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    InternalServerError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            message: self.to_string(),
        };

        match self {
            ApiError::BadRequest(_) => HttpResponse::BadRequest().json(error_response),
            ApiError::Unauthorized(_) => HttpResponse::Unauthorized().json(error_response),
            ApiError::NotFound(_) => HttpResponse::NotFound().json(error_response),
            ApiError::InternalServerError(_) => {
                HttpResponse::InternalServerError().json(error_response)
            }
        }
    }
} 