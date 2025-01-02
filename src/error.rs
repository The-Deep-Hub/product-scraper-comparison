use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use mongodb::error::Error as MongoError;
use redis::RedisError;
use lapin::Error as LapinError;
use serde_json::Error as JsonError;
use reqwest::Error as ReqwestError;
use std::error::Error as StdError;
use serde::Serialize;
use std::io;
use crate::domain::models::DomainError;

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Internal Server Error: {}", _0)]
    InternalServerError(String),
    #[display(fmt = "Not Found: {}", _0)]
    NotFound(String),
    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),
    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),
    #[display(fmt = "Cache Error: {}", _0)]
    CacheError(String),
    #[display(fmt = "Queue Error: {}", _0)]
    QueueError(String),
    #[display(fmt = "Scraper Error: {}", _0)]
    ScraperError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl StdError for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type) = match self {
            AppError::InternalServerError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error"),
            AppError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "not_found"),
            AppError::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Unauthorized(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::CacheError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "cache_error"),
            AppError::QueueError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "queue_error"),
            AppError::ScraperError(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "scraper_error"),
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        })
    }
}

impl From<MongoError> for AppError {
    fn from(error: MongoError) -> Self {
        AppError::InternalServerError(error.to_string())
    }
}

impl From<RedisError> for AppError {
    fn from(error: RedisError) -> Self {
        AppError::CacheError(error.to_string())
    }
}

impl From<LapinError> for AppError {
    fn from(error: LapinError) -> Self {
        AppError::QueueError(error.to_string())
    }
}

impl From<JsonError> for AppError {
    fn from(error: JsonError) -> Self {
        AppError::InternalServerError(error.to_string())
    }
}

impl From<ReqwestError> for AppError {
    fn from(error: ReqwestError) -> Self {
        AppError::ScraperError(error.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::InternalServerError(error.to_string())
    }
}

impl From<AppError> for io::Error {
    fn from(error: AppError) -> Self {
        io::Error::new(io::ErrorKind::Other, error.to_string())
    }
}

impl From<DomainError> for AppError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::Cache(msg) => AppError::CacheError(msg),
            DomainError::Queue(msg) => AppError::QueueError(msg),
            DomainError::Scraping(msg) => AppError::ScraperError(msg),
            DomainError::Http(msg) => AppError::InternalServerError(msg),
            DomainError::Validation(msg) => AppError::BadRequest(msg),
            DomainError::NotFound(msg) => AppError::NotFound(msg),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>; 