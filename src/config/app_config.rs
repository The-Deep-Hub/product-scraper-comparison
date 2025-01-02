use serde::Deserialize;
use config::ConfigError;
use tracing::warn;

use crate::domain::config::{DatabaseConfig, CacheConfig, QueueConfig, HttpConfig};
use crate::adapters::outbound::{
    mongodb::config::MongoConfig,
    cache::config::RedisConfig,
    queue::config::RabbitMQConfig,
    http::config::ZyteConfig,
};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default)]
    pub server: ServerConfig,
    pub database: MongoConfig,
    pub cache: RedisConfig,
    pub queue: RabbitMQConfig,
    pub http_client: ZyteConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl AppConfig {
    pub fn database(&self) -> &dyn DatabaseConfig {
        &self.database
    }

    pub fn cache(&self) -> &dyn CacheConfig {
        &self.cache
    }

    pub fn queue(&self) -> &dyn QueueConfig {
        &self.queue
    }

    pub fn http_client(&self) -> &dyn HttpConfig {
        &self.http_client
    }

    /// Gets the AMQP URL for RabbitMQ
    pub fn amqp_url(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}",
            self.queue.user,
            self.queue.password,
            self.queue.host,
            self.queue.port,
        )
    }

    /// Gets the Redis URL
    pub fn redis_url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}/",
            self.cache.user,
            self.cache.password,
            self.cache.host,
            self.cache.port,
        )
    }

    /// Gets the server address in host:port format
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Gets the Zyte API key
    pub fn zyte_api_key(&self) -> Option<String> {
        self.http_client.api_key()
    }

    /// Validates all configuration components
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate database configuration
        if self.database.connection_string().is_empty() {
            return Err(ConfigError::Message("Database connection string is empty".to_string()));
        }
        if self.database.database_name().is_empty() {
            return Err(ConfigError::Message("Database name is empty".to_string()));
        }

        // Validate cache configuration
        if self.cache.connection_url().is_empty() {
            return Err(ConfigError::Message("Cache connection URL is empty".to_string()));
        }

        // Validate queue configuration
        if self.queue.connection_url().is_empty() {
            return Err(ConfigError::Message("Queue connection URL is empty".to_string()));
        }
        if self.queue.queue_name().is_empty() {
            return Err(ConfigError::Message("Queue name is empty".to_string()));
        }

        // Validate HTTP client configuration
        if self.http_client.base_url().is_empty() {
            return Err(ConfigError::Message("HTTP client base URL is empty".to_string()));
        }
        if self.http_client.api_key().is_none() {
            warn!("HTTP client API key is not set");
        }

        // Validate server configuration
        if self.server.host.is_empty() {
            return Err(ConfigError::Message("Server host is empty".to_string()));
        }
        if self.server.port == 0 {
            return Err(ConfigError::Message("Server port is invalid".to_string()));
        }

        Ok(())
    }
} 