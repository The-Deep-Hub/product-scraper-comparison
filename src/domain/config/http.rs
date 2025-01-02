use std::time::Duration;

/// HTTP client configuration port
pub trait HttpConfig: Send + Sync {
    /// Get the base URL for the HTTP service
    fn base_url(&self) -> String;
    
    /// Get the API key if required
    fn api_key(&self) -> Option<String>;
    
    /// Get request timeout
    fn request_timeout(&self) -> Duration;
    
    /// Get maximum concurrent requests
    fn max_concurrent_requests(&self) -> u32;
    
    /// Get retry configuration
    fn max_retries(&self) -> u32;
    fn retry_delay(&self) -> Duration;
} 