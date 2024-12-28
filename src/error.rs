use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use mongodb::error::Error as MongoError;
use redis::RedisError;
use lapin::Error as LapinError;
use serde_json::Error as JsonError;
use reqwest::Error as ReqwestError;
use std::error::Error as StdError;

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
    #[display(fmt = "Worker Error: {}", _0)]
    WorkerError(String),
    #[display(fmt = "Configuration Error: {}", _0)]
    ConfigError(String),
    #[display(fmt = "Rate Limit Error: {}", _0)]
    RateLimitError(String),
    #[display(fmt = "Task Error: {}", _0)]
    TaskError(String),
}

impl StdError for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InternalServerError(msg) => HttpResponse::InternalServerError().json(msg),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(msg),
            AppError::BadRequest(msg) => HttpResponse::BadRequest().json(msg),
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().json(msg),
            AppError::CacheError(msg) => HttpResponse::InternalServerError().json(msg),
            AppError::QueueError(msg) => HttpResponse::InternalServerError().json(msg),
            AppError::ScraperError(msg) => HttpResponse::InternalServerError().json(msg),
            AppError::WorkerError(msg) => HttpResponse::InternalServerError().json(msg),
            AppError::ConfigError(msg) => HttpResponse::InternalServerError().json(msg),
            AppError::RateLimitError(msg) => HttpResponse::TooManyRequests().json(msg),
            AppError::TaskError(msg) => HttpResponse::InternalServerError().json(msg),
        }
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

pub type AppResult<T> = Result<T, AppError>; 