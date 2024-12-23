use serde::Deserialize;
use mongodb::{Client, Database};

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

impl MongoConfig {
    pub async fn connect(&self) -> mongodb::error::Result<Database> {
        let mut options = mongodb::options::ClientOptions::parse(&self.uri).await?;
        
        if let Some(min_size) = self.min_pool_size {
            options.min_pool_size = Some(min_size);
        }
        if let Some(max_size) = self.max_pool_size {
            options.max_pool_size = Some(max_size);
        }

        let client = Client::with_options(options)?;
        Ok(client.database(&self.database))
    }
} 