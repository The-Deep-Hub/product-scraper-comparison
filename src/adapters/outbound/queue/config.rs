use serde::Deserialize;

/// RabbitMQ adapter configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMQConfig {
    #[serde(default)]
    pub amqp_addr: String,
    #[serde(default = "default_prefetch_count")]
    pub prefetch_count: u16,
}

fn default_prefetch_count() -> u16 {
    1
}

impl Default for RabbitMQConfig {
    fn default() -> Self {
        Self {
            amqp_addr: String::new(),
            prefetch_count: default_prefetch_count(),
        }
    }
}

impl RabbitMQConfig {
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.amqp_addr.is_empty() {
            return Err("AMQP address is not configured".to_string());
        }
        Ok(())
    }
} 