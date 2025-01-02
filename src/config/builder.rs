use std::env;
use config::{Config, ConfigError, Environment, File};
use tracing::{warn, debug};

use super::app_config::AppConfig;

fn get_mongodb_config() -> (String, String) {
    // Try to get the full URI from environment variable
    let uri = if let Ok(uri) = env::var("MONGODB_URI") {
        uri
    } else {
        // Construct URI from individual components
        let username = env::var("MONGO_APP_USERNAME").unwrap_or_else(|_| "rust_scraper".to_string());
        let password = env::var("MONGO_APP_PASSWORD").unwrap_or_else(|_| "rust_scraper_password".to_string());
        let host = env::var("MONGO_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("MONGO_PORT").unwrap_or_else(|_| "27017".to_string());
        let database = env::var("MONGO_DATABASE").unwrap_or_else(|_| "rust_scraper".to_string());

        format!("mongodb://{}:{}@{}:{}/{}", username, password, host, port, database)
    };

    let database = env::var("MONGO_DATABASE").unwrap_or_else(|_| "rust_scraper".to_string());

    debug!("MongoDB configuration:");
    debug!("  URI: {}", uri);
    debug!("  Database: {}", database);

    (uri, database)
}

fn get_redis_config() -> (String, String, String, String) {
    // Try to get the full URL from environment variable
    if let Ok(url) = env::var("REDIS_URL") {
        debug!("Using Redis URL from environment: {}", url);
        let parts: Vec<&str> = url.split('@').collect();
        if parts.len() == 2 {
            let auth = parts[0].trim_start_matches("redis://:");
            let host_port = parts[1];
            let host = host_port.split(':').next().unwrap_or("localhost").to_string();
            let port = host_port.split(':').nth(1).unwrap_or("6379").trim_end_matches('/').to_string();
            
            debug!("Redis configuration (from URL):");
            debug!("  Host: {}", host);
            debug!("  Port: {}", port);
            debug!("  Auth: {}", auth);
            
            return (host, port, "default".to_string(), auth.to_string());
        }
    }

    // Construct from individual components
    let host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    let user = "default".to_string();
    let password = env::var("REDIS_PASSWORD").unwrap_or_else(|_| {
        warn!("REDIS_PASSWORD not found in environment variables, using default from .env");
        "redis_password123".to_string()
    });

    debug!("Redis configuration (from env):");
    debug!("  Host: {}", host);
    debug!("  Port: {}", port);
    debug!("  User: {}", user);
    debug!("  Password is set: {}", !password.is_empty());

    (host, port, user, password)
}

fn get_rabbitmq_config() -> (String, String, String, String, String) {
    // Try to get the full URL from environment variable
    if let Ok(url) = env::var("AMQP_ADDR") {
        debug!("Using AMQP URL from environment: {}", url);
        return (
            env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string()),
            env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string()),
            env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string()),
            env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string()),
            env::var("RABBITMQ_VHOST").unwrap_or_else(|_| "/".to_string()),
        );
    }

    let host = env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());
    let user = env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string());
    let password = env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string());
    let vhost = env::var("RABBITMQ_VHOST").unwrap_or_else(|_| "/".to_string());

    debug!("RabbitMQ configuration:");
    debug!("  Host: {}", host);
    debug!("  Port: {}", port);
    debug!("  User: {}", user);
    debug!("  VHost: {}", vhost);

    (host, port, user, password, vhost)
}

fn get_zyte_config() -> (String, String, u64, u64) {
    let api_key = env::var("ZYTE_API_KEY")
        .or_else(|_| env::var("ZYTE_SCRAPER_API_KEY"))
        .or_else(|_| env::var("SCRAPER_API_KEY"))
        .unwrap_or_else(|_| {
            warn!("No Zyte API key found in environment variables. Checking configuration files...");
            "c6b1e238a38c4baa8f8a299d8fdf8446".to_string() // Your API key from .env
        });

    if api_key.is_empty() {
        panic!("Zyte API key is empty. Please set ZYTE_API_KEY environment variable.");
    }

    let base_url = env::var("ZYTE_ENDPOINT")
        .unwrap_or_else(|_| "https://api.zyte.com/v1/extract".to_string());
    let timeout = env::var("ZYTE_REQUEST_TIMEOUT")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30) * 1000;
    let concurrent_requests = env::var("ZYTE_CONCURRENT_REQUESTS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(5);

    debug!("Zyte configuration:");
    debug!("  Base URL: {}", base_url);
    debug!("  API Key: {}", if api_key.is_empty() { "NOT SET" } else { "SET" });
    debug!("  Timeout: {}ms", timeout);
    debug!("  Concurrent requests: {}", concurrent_requests);

    (api_key, base_url, timeout, concurrent_requests)
}

pub fn new() -> Result<AppConfig, ConfigError> {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into());
    
    warn!("Loading configuration for environment: {}", environment);

    let (mongodb_uri, mongodb_database) = get_mongodb_config();
    let (zyte_api_key, zyte_base_url, zyte_timeout, zyte_concurrent_requests) = get_zyte_config();
    let (redis_host, redis_port, redis_user, redis_password) = get_redis_config();
    let (rabbitmq_host, rabbitmq_port, rabbitmq_user, rabbitmq_password, rabbitmq_vhost) = get_rabbitmq_config();

    let config = Config::builder()
        // Start with base configuration
        .add_source(File::with_name("config/base.yaml"))
        
        // Add environment-specific configuration
        .add_source(File::with_name(&format!("config/{}.yaml", environment)).required(false))
        
        // Add local configuration (not in version control)
        .add_source(File::with_name("config/local.yaml").required(false))
        
        // Add environment variables with prefix
        .add_source(
            Environment::with_prefix("APP")
                .separator("_")
                .try_parsing(true)
        )
        
        // Database configuration
        .set_override("database.uri", Some(mongodb_uri))?
        .set_override("database.database", Some(mongodb_database))?
        
        // HTTP client configuration
        .set_override("http_client.api_key", Some(zyte_api_key))?
        .set_override("http_client.base_url", Some(zyte_base_url))?
        .set_override("http_client.request_timeout_ms", Some(zyte_timeout))?
        .set_override("http_client.max_concurrent_requests", Some(zyte_concurrent_requests))?
        
        // Cache configuration
        .set_override("cache.host", Some(redis_host))?
        .set_override("cache.port", Some(redis_port))?
        .set_override("cache.user", Some(redis_user))?
        .set_override("cache.password", Some(redis_password))?
        
        // Queue configuration
        .set_override("queue.host", Some(rabbitmq_host))?
        .set_override("queue.port", Some(rabbitmq_port))?
        .set_override("queue.user", Some(rabbitmq_user))?
        .set_override("queue.password", Some(rabbitmq_password))?
        .set_override("queue.vhost", Some(rabbitmq_vhost))?
        
        .build()?;

    // Deserialize the configuration
    let app_config: AppConfig = config.try_deserialize()?;
    
    warn!("Validating configuration");
    app_config.validate()?;

    Ok(app_config)
}
