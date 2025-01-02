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
        let api_key = std::env::var("ZYTE_API_KEY")
            .or_else(|_| std::env::var("ZYTE_SCRAPER_API_KEY"))
            .or_else(|_| std::env::var("SCRAPER_API_KEY"))
            .unwrap_or_default();

        Self {
            api_key,
            endpoint: default_endpoint(),
            concurrent_requests: default_concurrent_requests(),
            request_timeout: default_request_timeout(),
        }
    }
}

impl ZyteConfig {
    /// Validates the configuration
    pub fn validate(&mut self) -> Result<(), String> {
        let api_key = if self.api_key.is_empty() {
            std::env::var("ZYTE_API_KEY")
                .or_else(|_| std::env::var("ZYTE_SCRAPER_API_KEY"))
                .or_else(|_| std::env::var("SCRAPER_API_KEY"))
                .unwrap_or_default()
        } else {
            self.api_key.clone()
        };

        if api_key.is_empty() {
            return Err("Zyte API key is not configured. Please set either ZYTE_API_KEY, ZYTE_SCRAPER_API_KEY, or SCRAPER_API_KEY environment variable.".to_string());
        }
        self.api_key = api_key;
        Ok(())
    }
} 