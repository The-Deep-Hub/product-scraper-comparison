use serde::Deserialize;
use tracing::warn;
use crate::config::error::ConfigError;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: u32,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u32,
}

fn default_endpoint() -> String { "https://api.zyte.com/v1/extract".to_string() }
fn default_concurrent_requests() -> u32 { 5 }
fn default_request_timeout() -> u32 { 30 }

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.api_key.is_empty() {
            warn!("Zyte API key is not set");
        }
        Ok(())
    }
}
