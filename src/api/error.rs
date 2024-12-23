use actix_web::{
    error::ResponseError,
    http::StatusCode,
    HttpResponse,
};
use derive_more::Display;
use serde::Serialize;
use serde_json::json;

use crate::db::error::DbError;

#[derive(Debug, Display, Serialize)]
#[serde(untagged)]
pub enum ApiError {
    #[display(fmt = "Not Found: {}", _0)]
    NotFound(String),
    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),
    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),
    #[display(fmt = "Internal Server Error: {}", _0)]
    InternalServerError(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let message = self.to_string();
        HttpResponse::build(status).json(json!({ "error": message }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<DbError> for ApiError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound(msg) => ApiError::NotFound(msg),
            DbError::ValidationError(msg) => ApiError::BadRequest(msg),
            DbError::MongoError(err) => ApiError::InternalServerError(err.to_string()),
            DbError::InternalError(msg) => ApiError::InternalServerError(msg),
        }
    }
} 