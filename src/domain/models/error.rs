use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Scraping error: {0}")]
    Scraping(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Queue error: {0}")]
    Queue(String),
    
    #[error("HTTP client error: {0}")]
    HttpClient(String),
}

impl DomainError {
    pub fn validation<T: fmt::Display>(msg: T) -> Self {
        Self::Validation(msg.to_string())
    }

    pub fn not_found<T: fmt::Display>(msg: T) -> Self {
        Self::NotFound(msg.to_string())
    }
} 