use std::fmt;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug)]
pub enum CoreError {
    ServiceInitialization(String),
    Configuration(String),
    Database(String),
    External(String),
}

impl std::error::Error for CoreError {}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CoreError::ServiceInitialization(msg) => write!(f, "Service initialization error: {}", msg),
            CoreError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            CoreError::Database(msg) => write!(f, "Database error: {}", msg),
            CoreError::External(msg) => write!(f, "External service error: {}", msg),
        }
    }
}
