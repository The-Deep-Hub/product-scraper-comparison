use serde::Deserialize;

/// Zyte adapter configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ZyteConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: u32,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u32,
}

fn default_endpoint() -> String {
    "https://api.zyte.com/v1/extract".to_string()
}

fn default_concurrent_requests() -> u32 {
    5
}

fn default_request_timeout() -> u32 {
    30
}

impl Default for ZyteConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            endpoint: default_endpoint(),
            concurrent_requests: default_concurrent_requests(),
            request_timeout: default_request_timeout(),
        }
    }
}

impl ZyteConfig {
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("Zyte API key is not configured".to_string());
        }
        Ok(())
    }
} 