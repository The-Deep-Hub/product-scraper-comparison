use std::path::Path;
use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use tracing::{info, warn, debug};

use crate::config::services::{
    DatabaseConfig,
    QueueConfig,
    ServerConfig,
    ZyteConfig,
    WorkerConfig,
};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub queue: QueueConfig,
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
            database: DatabaseConfig::default(),
            queue: QueueConfig::default(),
            zyte: ZyteConfig::default(),
            worker: WorkerConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Load .env file
        dotenv().ok();

        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "development".into());
        info!("Loading configuration for environment: {}", env);

        let config_dir = Path::new("config");
        
        let mut config = Config::builder()
            // Start with base configuration
            .add_source(File::from(config_dir.join("base.yaml")))
            // Add environment specific configuration if not in development
            .add_source(
                File::from(config_dir.join(format!("{}.yaml", env)))
                    .required(env != "development")
            )
            // Add environment variables without prefix
            .add_source(Environment::default().separator("_").ignore_empty(true))
            .build()?;

        // Ensure server configuration has default values
        if config.get_string("server.host").is_err() {
            config.set("server.host", "0.0.0.0")?;
        }
        if config.get_int("server.port").is_err() {
            config.set("server.port", 8080i64)?;
        }

        // Construct MongoDB URI from individual environment variables if not set
        if config.get_string("database.mongodb_uri").unwrap_or_default().is_empty() {
            let username = std::env::var("MONGO_APP_USERNAME").unwrap_or_default();
            let password = std::env::var("MONGO_APP_PASSWORD").unwrap_or_default();
            let host = std::env::var("MONGO_HOST").unwrap_or_default();
            let port = std::env::var("MONGO_PORT").unwrap_or_default();
            let database = std::env::var("MONGO_DATABASE").unwrap_or_default();

            if !username.is_empty() && !password.is_empty() && !host.is_empty() && !port.is_empty() && !database.is_empty() {
                let mongodb_uri = format!(
                    "mongodb://{}:{}@{}:{}/{}",
                    username, password, host, port, database
                );
                config.set("database.mongodb_uri", mongodb_uri)?;
            }
        }

        // Set Redis configuration from environment variables
        if let Ok(redis_host) = std::env::var("REDIS_HOST") {
            config.set("database.redis_host", redis_host)?;
        }
        if let Ok(redis_port) = std::env::var("REDIS_PORT") {
            if let Ok(port) = redis_port.parse::<u16>() {
                config.set("database.redis_port", port)?;
            }
        }
        if let Ok(redis_password) = std::env::var("REDIS_PASSWORD") {
            config.set("database.redis_password", redis_password)?;
        }

        // Set AMQP configuration
        if let Ok(amqp_addr) = std::env::var("AMQP_ADDR") {
            config.set("queue.amqp_addr", amqp_addr)?;
        }

        // Set Zyte configuration
        if let Ok(zyte_api_key) = std::env::var("ZYTE_API_KEY") {
            config.set("zyte.api_key", zyte_api_key)?;
        }
        if let Ok(zyte_endpoint) = std::env::var("ZYTE_ENDPOINT") {
            config.set("zyte.endpoint", zyte_endpoint)?;
        }
        if let Ok(concurrent_requests) = std::env::var("ZYTE_CONCURRENT_REQUESTS") {
            if let Ok(requests) = concurrent_requests.parse::<u32>() {
                config.set("zyte.concurrent_requests", requests)?;
            }
        }
        if let Ok(request_timeout) = std::env::var("ZYTE_REQUEST_TIMEOUT") {
            if let Ok(timeout) = request_timeout.parse::<u32>() {
                config.set("zyte.request_timeout", timeout)?;
            }
        }

        // Set worker configuration
        if let Ok(prefetch_count) = std::env::var("WORKER_PREFETCH_COUNT") {
            if let Ok(count) = prefetch_count.parse::<u16>() {
                config.set("worker.prefetch_count", count as i64)?;
            }
        }
        if let Ok(reconnect_delay) = std::env::var("WORKER_RECONNECT_DELAY") {
            if let Ok(delay) = reconnect_delay.parse::<u64>() {
                config.set("worker.reconnect_delay_secs", delay as i64)?;
            }
        }
        if let Ok(product_limit) = std::env::var("WORKER_PRODUCT_LIMIT") {
            if let Ok(limit) = product_limit.parse::<usize>() {
                config.set("worker.product_limit", limit as i64)?;
            }
        }

        let config = config.try_deserialize()?;
        debug!("Final configuration: {:?}", config);
        Self::validate_config(&config)?;
        Ok(config)
    }

    fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
        // Check required environment variables
        if config.database.redis_password.is_empty() {
            warn!("REDIS_PASSWORD environment variable is not set");
            return Err(ConfigError::NotFound("REDIS_PASSWORD".into()));
        }

        if config.database.mongodb_uri.is_empty() {
            warn!("MongoDB connection details are not properly configured");
            return Err(ConfigError::NotFound("MongoDB connection details".into()));
        }

        if config.queue.amqp_addr.is_empty() {
            warn!("AMQP_ADDR environment variable is not set");
            return Err(ConfigError::NotFound("AMQP_ADDR".into()));
        }

        if config.zyte.api_key.is_empty() {
            warn!("ZYTE_API_KEY environment variable is not set");
            return Err(ConfigError::NotFound("ZYTE_API_KEY".into()));
        }

        Ok(())
    }

    pub fn amqp_url(&self) -> String {
        self.queue.amqp_addr.clone()
    }

    pub fn redis_url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}/",
            self.database.redis_user,
            self.database.redis_password,
            self.database.redis_host,
            self.database.redis_port,
        )
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
