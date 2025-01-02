use serde::Deserialize;
use std::time::Duration;
use crate::domain::config::DatabaseConfig;

/// MongoDB configuration
#[derive(Debug, Clone, Deserialize)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
    pub min_pool_size: Option<u32>,
    pub max_pool_size: Option<u32>,
    pub connection_timeout_ms: Option<u64>,
}

impl Default for MongoConfig {
    fn default() -> Self {
        Self {
            uri: "mongodb://localhost:27017".to_string(),
            database: "rust_scraper".to_string(),
            min_pool_size: Some(5),
            max_pool_size: Some(10),
            connection_timeout_ms: Some(5000),
        }
    }
}

impl DatabaseConfig for MongoConfig {
    fn connection_string(&self) -> String {
        self.uri.clone()
    }
    
    fn database_name(&self) -> String {
        self.database.clone()
    }
    
    fn connection_timeout(&self) -> Duration {
        Duration::from_millis(self.connection_timeout_ms.unwrap_or(5000))
    }
    
    fn max_pool_size(&self) -> u32 {
        self.max_pool_size.unwrap_or(10)
    }
} 