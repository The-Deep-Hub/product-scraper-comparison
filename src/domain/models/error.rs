use std::fmt;

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug)]
pub enum DomainError {
    NotFound(String),
    Validation(String),
    Scraping(String),
    Cache(String),
    Queue(String),
    Http(String),
}

impl DomainError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn scraping(msg: impl Into<String>) -> Self {
        Self::Scraping(msg.into())
    }

    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    pub fn queue(msg: impl Into<String>) -> Self {
        Self::Queue(msg.into())
    }

    pub fn http(msg: impl Into<String>) -> Self {
        Self::Http(msg.into())
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::Scraping(msg) => write!(f, "Scraping error: {}", msg),
            Self::Cache(msg) => write!(f, "Cache error: {}", msg),
            Self::Queue(msg) => write!(f, "Queue error: {}", msg),
            Self::Http(msg) => write!(f, "HTTP error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {} 