use serde::Deserialize;

/// MongoDB configuration
#[derive(Debug, Clone, Deserialize)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
    pub min_pool_size: Option<u32>,
    pub max_pool_size: Option<u32>,
}

impl Default for MongoConfig {
    fn default() -> Self {
        Self {
            uri: "mongodb://localhost:27017".to_string(),
            database: "rust_scraper".to_string(),
            min_pool_size: Some(5),
            max_pool_size: Some(10),
        }
    }
} 