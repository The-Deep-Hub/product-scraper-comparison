use serde::Deserialize;
use std::time::Duration;
use crate::domain::config::HttpConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct ZyteConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub request_timeout_ms: Option<u64>,
    pub max_concurrent_requests: Option<u32>,
    pub max_retries: Option<u32>,
    pub retry_delay_ms: Option<u64>,
}

impl Default for ZyteConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.zyte.com/v1".to_string(),
            api_key: None,
            request_timeout_ms: Some(30000),
            max_concurrent_requests: Some(5),
            max_retries: Some(3),
            retry_delay_ms: Some(1000),
        }
    }
}

impl HttpConfig for ZyteConfig {
    fn base_url(&self) -> String {
        self.base_url.clone()
    }
    
    fn api_key(&self) -> Option<String> {
        self.api_key.clone()
    }
    
    fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.request_timeout_ms.unwrap_or(30000))
    }
    
    fn max_concurrent_requests(&self) -> u32 {
        self.max_concurrent_requests.unwrap_or(5)
    }
    
    fn max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }
    
    fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_ms.unwrap_or(1000))
    }
} 