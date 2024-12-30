use std::path::Path;
use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use tracing::{info, warn, debug};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DatabaseConfig {
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

fn default_redis_host() -> String {
    "localhost".to_string()
}

fn default_redis_port() -> u16 {
    6379
}

fn default_redis_user() -> String {
    "default".to_string()
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct QueueConfig {
    #[serde(default)]
    pub amqp_addr: String,
    #[serde(default = "default_prefetch_count")]
    pub prefetch_count: u16,
}

fn default_prefetch_count() -> u16 {
    1
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ZyteConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_zyte_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_zyte_concurrent_requests")]
    pub concurrent_requests: u32,
    #[serde(default = "default_zyte_request_timeout")]
    pub request_timeout: u32,
}

fn default_zyte_endpoint() -> String {
    "https://api.zyte.com/v1/extract".to_string()
}

fn default_zyte_concurrent_requests() -> u32 {
    5
}

fn default_zyte_request_timeout() -> u32 {
    30
}

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
