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
    
    // Start building configuration
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
    
    // Set default values for all required fields
    debug!("Setting default values");

    // Server defaults
    builder = builder
        .set_default("server.host", "0.0.0.0")?
        .set_default("server.port", 8080i64)?;

    // MongoDB defaults
    if let Ok(mongodb_uri) = std::env::var("MONGO_APP_USERNAME")
        .and_then(|username| {
            std::env::var("MONGO_APP_PASSWORD").map(|password| (username, password))
        })
        .and_then(|(username, password)| {
            std::env::var("MONGO_HOST")
                .and_then(|host| std::env::var("MONGO_PORT").map(|port| (host, port)))
                .map(|(host, port)| {
                    format!(
                        "mongodb://{}:{}@{}:{}/{}",
                        username, password, host, port,
                        std::env::var("MONGO_DATABASE").unwrap_or_default()
                    )
                })
        }) {
        builder = builder.set_default("mongodb.uri", mongodb_uri)?;
    }

    builder = builder
        .set_default("mongodb.database", "rust_scraper")?
        .set_default("mongodb.min_pool_size", 5i64)?
        .set_default("mongodb.max_pool_size", 10i64)?;

    // Redis defaults
    builder = builder
        .set_default("redis.host", "localhost")?
        .set_default("redis.port", 6379i64)?
        .set_default("redis.user", "default")?;

    if let Ok(password) = std::env::var("REDIS_PASSWORD") {
        builder = builder.set_default("redis.password", password)?;
    }

    // RabbitMQ defaults
    let amqp_addr = std::env::var("AMQP_ADDR")
        .map(|addr| {
            // Ensure URL starts with amqp://
            let addr = if !addr.starts_with("amqp://") {
                format!("amqp://{}", addr)
            } else {
                addr
            };

            // Parse the URL to check for vhost
            let without_protocol = addr.trim_start_matches("amqp://");
            if let Some(host_part) = without_protocol.split('@').nth(1) {
                let host_parts: Vec<&str> = host_part.split('/').collect();
                match host_parts.len() {
                    1 => format!("{}/%2F", addr),
                    2 if host_parts[1].is_empty() => format!("{}%2F", addr),
                    2 => addr,
                    _ => addr.split('/').take(4).collect::<Vec<&str>>().join("/"),
                }
            } else {
                format!("{}/%2F", addr)
            }
        })
        .or_else(|_: std::env::VarError| {
            // Try to construct from individual components
            let host = std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port = std::env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());
            let user = std::env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string());
            let pass = std::env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string());
            let vhost = std::env::var("RABBITMQ_VHOST")
                .unwrap_or_else(|_| "/".to_string())
                .trim_matches('/')
                .to_string();
            
            let vhost_part = if vhost.is_empty() || vhost == "/" {
                "%2F"
            } else {
                &vhost
            };

            Ok::<String, std::env::VarError>(format!("amqp://{}:{}@{}:{}/{}", user, pass, host, port, vhost_part))
        })
        .or_else(|_: std::env::VarError| std::env::var("RABBITMQ_AMQP_ADDR"))
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2F".to_string());
    
    builder = builder
        .set_default("rabbitmq.amqp_addr", amqp_addr)?
        .set_default("rabbitmq.prefetch_count", 1i64)?;

    // Zyte defaults
    let zyte_api_key = std::env::var("ZYTE_API_KEY")
        .or_else(|_| std::env::var("ZYTE_SCRAPER_API_KEY"))
        .or_else(|_| std::env::var("SCRAPER_API_KEY"))
        .unwrap_or_default();

    builder = builder
        .set_default("zyte.api_key", zyte_api_key.clone())?
        .set_default("zyte.endpoint", "https://api.zyte.com/v1/extract")?
        .set_default("zyte.concurrent_requests", 5i64)?
        .set_default("zyte.request_timeout", 30i64)?;

    // Set store-specific API keys if not already set
    for store in ["leroy", "bauhaus", "bricodepot"] {
        builder = builder.set_default(
            &format!("stores.{}.api.api_key", store),
            zyte_api_key.clone()
        )?;
    }

    // Worker defaults
    builder = builder
        .set_default("worker.prefetch_count", 3i64)?
        .set_default("worker.reconnect_delay_secs", 5i64)?
        .set_default("worker.product_limit", 100i64)?;

    // Set store queues defaults
    let mut queues = std::collections::HashMap::new();
    queues.insert("leroy".to_string(), "leroy_tasks".to_string());
    queues.insert("bauhaus".to_string(), "bauhaus_tasks".to_string());
    queues.insert("bricodepot".to_string(), "bricodepot_tasks".to_string());
    builder = builder.set_default("worker.store_queues", queues)?;

    // Build the configuration
    debug!("Building configuration");
    let config = builder.build()?;
    
    // Try to deserialize the configuration
    debug!("Attempting to deserialize configuration");
    let config_str = format!("{:?}", config);
    debug!("Configuration before deserialization: {}", config_str);
    
    let mut app_config: AppConfig = match config.try_deserialize() {
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
