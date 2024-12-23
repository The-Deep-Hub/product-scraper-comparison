use mongodb::error::Error as MongoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("MongoDB error: {0}")]
    MongoError(#[from] MongoError),
    #[error("Document not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<String> for DbError {
    fn from(error: String) -> Self {
        DbError::InternalError(error)
    }
}

impl From<&str> for DbError {
    fn from(error: &str) -> Self {
        DbError::InternalError(error.to_string())
    }
} 