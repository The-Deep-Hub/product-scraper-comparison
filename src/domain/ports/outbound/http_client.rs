use async_trait::async_trait;
use std::collections::HashMap;
use crate::domain::models::DomainError;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[async_trait]
pub trait HttpClientPort: Send + Sync {
    /// Get rendered HTML from a URL (with JavaScript execution)
    async fn get_rendered_html(&self, url: &str) -> Result<String, DomainError>;
    
    /// Make a GET request
    async fn get(&self, url: &str) -> Result<HttpResponse, DomainError>;
    
    /// Make a POST request
    async fn post(&self, url: &str, body: &str) -> Result<HttpResponse, DomainError>;
    
    /// Set default headers for all requests
    async fn set_default_headers(&self, headers: HashMap<String, String>) -> Result<(), DomainError>;
} 