use serde::Deserialize;
use config::ConfigError;
use tracing::warn;

use crate::adapters::{
    outbound::{
        mongodb::config::MongoConfig,
        cache::config::RedisConfig,
        queue::config::RabbitMQConfig,
        http::config::ZyteConfig,
    },
    inbound::{
        api::config::ServerConfig,
        worker::config::WorkerConfig,
    },
};

/// Main application configuration that aggregates all adapter configs
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub mongodb: MongoConfig,
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub rabbitmq: RabbitMQConfig,
    #[serde(default)]
    pub zyte: ZyteConfig,
    #[serde(default)]
    pub worker: WorkerConfig,
}

fn default_environment() -> String {
    "development".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: default_environment(),
            server: ServerConfig::default(),
            mongodb: MongoConfig::default(),
            redis: RedisConfig::default(),
            rabbitmq: RabbitMQConfig::default(),
            zyte: ZyteConfig::default(),
            worker: WorkerConfig::default(),
        }
    }
}

impl AppConfig {
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate Redis configuration
        if let Err(err) = self.redis.validate() {
            warn!("Redis configuration validation failed: {}", err);
            return Err(ConfigError::Message(err));
        }

        // Validate RabbitMQ configuration
        if let Err(err) = self.rabbitmq.validate() {
            warn!("RabbitMQ configuration validation failed: {}", err);
            return Err(ConfigError::Message(err));
        }

        // Validate Zyte configuration
        if let Err(err) = self.zyte.validate() {
            warn!("Zyte configuration validation failed: {}", err);
            return Err(ConfigError::Message(err));
        }

        Ok(())
    }

    /// Gets the AMQP URL for RabbitMQ
    pub fn amqp_url(&self) -> String {
        self.rabbitmq.amqp_addr.clone()
    }

    /// Gets the Redis URL
    pub fn redis_url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}/",
            self.redis.user,
            self.redis.password,
            self.redis.host,
            self.redis.port,
        )
    }

    /// Gets the server address in host:port format
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
} 