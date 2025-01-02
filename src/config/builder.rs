use std::path::Path;
use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use tracing::{info, warn, debug};

use super::AppConfig;

/// Creates a new AppConfig instance from environment variables and config files
pub fn new() -> Result<AppConfig, ConfigError> {
    // Load .env file
    dotenv().ok();

    let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "development".into());
    info!("Loading configuration for environment: {}", env);

    let config_dir = Path::new("config");
    debug!("Config directory: {:?}", config_dir);
    
    let base_file = config_dir.join("base.yaml");
    debug!("Base config file: {:?}", base_file);
    
    let env_file = config_dir.join(format!("{}.yaml", env));
    debug!("Environment config file: {:?}", env_file);
    
    let mut builder = Config::builder();
    
    // Add base configuration
    debug!("Loading base configuration");
    builder = builder.add_source(File::from(base_file));
    
    // Add environment specific configuration
    if env != "development" {
        debug!("Loading environment configuration");
        builder = builder.add_source(File::from(env_file).required(true));
    }
    
    // Add environment variables
    debug!("Adding environment variables");
    builder = builder.add_source(Environment::default().separator("_").ignore_empty(true));
    
    debug!("Building configuration");
    let mut config = builder.build()?;

    // Set default values for all required fields
    debug!("Setting default values");

    // Server defaults
    if config.get_string("server.host").is_err() {
        config.set("server.host", "0.0.0.0")?;
    }
    if config.get_int("server.port").is_err() {
        config.set("server.port", 8080i64)?;
    }

    // MongoDB defaults
    if config.get_string("mongodb.uri").unwrap_or_default().is_empty() {
        debug!("Constructing MongoDB URI from environment variables");
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
            debug!("Setting MongoDB URI");
            config.set("mongodb.uri", mongodb_uri)?;
        }
    }
    if config.get_string("mongodb.database").is_err() {
        config.set("mongodb.database", "rust_scraper")?;
    }
    if config.get_int("mongodb.min_pool_size").is_err() {
        config.set("mongodb.min_pool_size", 5i64)?;
    }
    if config.get_int("mongodb.max_pool_size").is_err() {
        config.set("mongodb.max_pool_size", 10i64)?;
    }

    // Redis defaults
    if config.get_string("redis.host").is_err() {
        config.set("redis.host", "localhost")?;
    }
    if config.get_int("redis.port").is_err() {
        config.set("redis.port", 6379i64)?;
    }
    if config.get_string("redis.user").is_err() {
        config.set("redis.user", "default")?;
    }
    if config.get_string("redis.password").is_err() {
        if let Ok(password) = std::env::var("REDIS_PASSWORD") {
            config.set("redis.password", password)?;
        }
    }

    // RabbitMQ defaults
    if config.get_string("rabbitmq.amqp_addr").is_err() {
        if let Ok(addr) = std::env::var("AMQP_ADDR") {
            config.set("rabbitmq.amqp_addr", addr)?;
        }
    }
    if config.get_int("rabbitmq.prefetch_count").is_err() {
        config.set("rabbitmq.prefetch_count", 1i64)?;
    }

    // Zyte defaults
    if config.get_string("zyte.api_key").is_err() {
        if let Ok(key) = std::env::var("ZYTE_API_KEY") {
            config.set("zyte.api_key", key)?;
        }
    }
    if config.get_string("zyte.endpoint").is_err() {
        config.set("zyte.endpoint", "https://api.zyte.com/v1/extract")?;
    }
    if config.get_int("zyte.concurrent_requests").is_err() {
        config.set("zyte.concurrent_requests", 5i64)?;
    }
    if config.get_int("zyte.request_timeout").is_err() {
        config.set("zyte.request_timeout", 30i64)?;
    }

    // Worker defaults
    if config.get_int("worker.prefetch_count").is_err() {
        config.set("worker.prefetch_count", 3i64)?;
    }
    if config.get_int("worker.reconnect_delay_secs").is_err() {
        config.set("worker.reconnect_delay_secs", 5i64)?;
    }
    if config.get_int("worker.product_limit").is_err() {
        config.set("worker.product_limit", 100i64)?;
    }
    if config.get_table("worker.store_queues").is_err() {
        let mut queues = std::collections::HashMap::new();
        queues.insert("leroy".to_string(), "leroy_tasks".to_string());
        queues.insert("bauhaus".to_string(), "bauhaus_tasks".to_string());
        queues.insert("bricodepot".to_string(), "bricodepot_tasks".to_string());
        config.set("worker.store_queues", queues)?;
    }

    // Try to deserialize the configuration
    debug!("Attempting to deserialize configuration");
    let config_str = format!("{:?}", config);
    debug!("Configuration before deserialization: {}", config_str);
    
    let app_config: AppConfig = match config.try_deserialize() {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("Failed to deserialize configuration: {}", e);
            return Err(e);
        }
    };

    // Validate the configuration
    debug!("Validating configuration");
    app_config.validate()?;

    Ok(app_config)
}
