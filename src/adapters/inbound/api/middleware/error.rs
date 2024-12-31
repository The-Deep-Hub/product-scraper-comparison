use actix_web::{
    error::ResponseError,
    http::StatusCode,
    HttpResponse,
};
use serde::Serialize;
use crate::domain::models::DomainError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    code: u16,
    message: String,
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl ResponseError for DomainError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            DomainError::NotFound(_) => StatusCode::NOT_FOUND,
            DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let response = ErrorResponse {
            code: status.as_u16(),
            message: self.to_string(),
        };

        HttpResponse::build(status).json(response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            DomainError::NotFound(_) => StatusCode::NOT_FOUND,
            DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
} 