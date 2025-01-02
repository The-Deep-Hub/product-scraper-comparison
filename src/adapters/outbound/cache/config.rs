use serde::Deserialize;
use std::time::Duration;
use crate::domain::config::CacheConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub default_ttl_secs: Option<u64>,
    pub connection_timeout_ms: Option<u64>,
    pub max_connections: Option<u32>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6379,
            user: "default".to_string(),
            password: "password".to_string(),
            default_ttl_secs: Some(3600),
            connection_timeout_ms: Some(5000),
            max_connections: Some(10),
        }
    }
}

impl CacheConfig for RedisConfig {
    fn connection_url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}/",
            self.user,
            self.password,
            self.host,
            self.port,
        )
    }
    
    fn default_ttl(&self) -> Duration {
        Duration::from_secs(self.default_ttl_secs.unwrap_or(3600))
    }
    
    fn connection_timeout(&self) -> Duration {
        Duration::from_millis(self.connection_timeout_ms.unwrap_or(5000))
    }
    
    fn max_connections(&self) -> u32 {
        self.max_connections.unwrap_or(10)
    }
} 