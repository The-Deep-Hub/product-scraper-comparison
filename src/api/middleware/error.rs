use actix_web::HttpResponse;
use serde::Serialize;

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