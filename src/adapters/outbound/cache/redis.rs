use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::info;

use crate::domain::models::{DomainError, DomainResult, Product};
use crate::domain::ports::outbound::CachePort;

const CACHE_EXPIRY: u64 = 3600; // 1 hour

#[derive(Clone)]
pub struct RedisAdapter {
    connection: ConnectionManager,
}

impl RedisAdapter {
    pub async fn new() -> Result<Self, DomainError> {
        let redis_url = format!(
            "redis://{}:{}@{}:{}/",
            std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
            std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
            std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
        );
        
        info!("Connecting to Redis at: {}", redis_url.replace(|c: char| c.is_ascii_alphanumeric(), "*"));
        let client = redis::Client::open(redis_url)
            .map_err(|e| DomainError::cache(format!("Failed to create Redis client: {}", e)))?;
            
        let connection = client.get_tokio_connection_manager()
            .await
            .map_err(|e| DomainError::cache(format!("Failed to get Redis connection: {}", e)))?;
            
        info!("Successfully connected to Redis");
        Ok(Self { connection })
    }
    
    fn get_products_key(query: &str) -> String {
        format!("products:{}", query)
    }
}

#[async_trait]
impl CachePort for RedisAdapter {
    async fn get_products(&self, query: &str) -> DomainResult<Vec<Product>> {
        let mut conn = self.connection.clone();
        let cache_key = Self::get_products_key(query);
        
        let result: Option<String> = conn.get(&cache_key).await
            .map_err(|e| DomainError::cache(format!("Failed to get products from Redis: {}", e)))?;
            
        match result {
            Some(json) => {
                let products: Vec<Product> = serde_json::from_str(&json)
                    .map_err(|e| DomainError::cache(format!("Failed to deserialize products: {}", e)))?;
                Ok(products)
            }
            None => Ok(Vec::new())
        }
    }

    async fn cache_products(&self, query: &str, products: &[Product]) -> DomainResult<()> {
        let mut conn = self.connection.clone();
        let cache_key = Self::get_products_key(query);
        
        let json = serde_json::to_string(products)
            .map_err(|e| DomainError::cache(format!("Failed to serialize products: {}", e)))?;
            
        let _: () = conn.set_ex(&cache_key, json, CACHE_EXPIRY as usize).await
            .map_err(|e| DomainError::cache(format!("Failed to cache products in Redis: {}", e)))?;
            
        Ok(())
    }

    async fn invalidate(&self, query: &str) -> DomainResult<()> {
        let mut conn = self.connection.clone();
        let cache_key = Self::get_products_key(query);
        
        let _: () = conn.del(&cache_key).await
            .map_err(|e| DomainError::cache(format!("Failed to invalidate cache in Redis: {}", e)))?;
            
        Ok(())
    }
} 