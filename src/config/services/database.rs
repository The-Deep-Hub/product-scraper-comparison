use serde::Deserialize;
use tracing::warn;
use crate::config::error::ConfigError;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub mongodb_uri: String,
    #[serde(default = "default_redis_host")]
    pub redis_host: String,
    #[serde(default = "default_redis_port")]
    pub redis_port: u16,
    #[serde(default = "default_redis_user")]
    pub redis_user: String,
    #[serde(default)]
    pub redis_password: String,
}

fn default_redis_host() -> String { "localhost".to_string() }
fn default_redis_port() -> u16 { 6379 }
fn default_redis_user() -> String { "default".to_string() }

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.mongodb_uri.is_empty() {
            // For development, we'll just warn about missing MongoDB URI
            if cfg!(debug_assertions) {
                warn!("MongoDB URI is not set. Some features may not work properly.");
            } else {
                return Err(ConfigError::Missing("mongodb_uri".to_string()));
            }
        }
        Ok(())
    }

    pub fn redis_url(&self) -> String {
        if self.redis_password.is_empty() {
            format!(
                "redis://{}:{}", 
                self.redis_host, 
                self.redis_port
            )
        } else {
            format!(
                "redis://{}:{}@{}:{}",
                self.redis_user, 
                self.redis_password, 
                self.redis_host, 
                self.redis_port
            )
        }
    }
}
