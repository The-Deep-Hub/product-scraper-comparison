use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("MongoDB error: {0}")]
    MongoError(#[from] mongodb::error::Error),

    #[error("BSON serialization error: {0}")]
    BsonError(#[from] bson::ser::Error),

    #[error("BSON deserialization error: {0}")]
    BsonDeError(#[from] bson::de::Error),

    #[error("Document not found")]
    NotFound,

    #[error("Duplicate key error: {0}")]
    DuplicateKey(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    Other(String),
}

impl From<mongodb::error::Error> for DbError {
    fn from(error: mongodb::error::Error) -> Self {
        match error.kind.as_ref() {
            mongodb::error::ErrorKind::Write(write_error) => {
                if write_error.code == 11000 {
                    DbError::DuplicateKey(write_error.message.clone())
                } else {
                    DbError::MongoError(error)
                }
            }
            _ => DbError::MongoError(error),
        }
    }
} 