use serde::Deserialize;
use tracing::warn;
use crate::config::error::ConfigError;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub amqp_addr: String,
    #[serde(default = "default_prefetch_count")]
    pub prefetch_count: u16,
}

fn default_prefetch_count() -> u16 { 1 }

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.amqp_addr.is_empty() {
            // For development, we'll just warn about missing AMQP address
            if cfg!(debug_assertions) {
                warn!("AMQP address is not set. Queue functionality will not work properly.");
            } else {
                return Err(ConfigError::Missing("amqp_addr".to_string()));
            }
        }
        Ok(())
    }

    pub fn get_amqp_addr(&self) -> String {
        if self.amqp_addr.is_empty() {
            // Default development configuration
            "amqp://guest:guest@localhost:5672/%2f".to_string()
        } else {
            self.amqp_addr.clone()
        }
    }
}
