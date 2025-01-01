use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::{info, error};

use crate::domain::{
    models::{Product, DomainError},
    ports::outbound::CachePort,
};

pub struct RedisAdapter {
    client: ConnectionManager,
}

impl RedisAdapter {
    pub async fn new() -> Result<Self, DomainError> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        info!("Connecting to Redis at: {}", "*".repeat(redis_url.len()));
        
        let client = redis::Client::open(redis_url)
            .map_err(|e| DomainError::cache(format!("Failed to create Redis client: {}", e)))?
            .get_tokio_connection_manager()
            .await
            .map_err(|e| DomainError::cache(format!("Failed to get Redis connection: {}", e)))?;
        
        info!("Successfully connected to Redis");
        Ok(Self { client })
    }
}

#[async_trait]
impl CachePort for RedisAdapter {
    async fn get_products(&self, key: &str) -> Result<Vec<Product>, DomainError> {
        let mut conn = self.client.clone();
        let data: Option<String> = conn.get(key).await
            .map_err(|e| DomainError::cache(format!("Failed to get products from Redis: {}", e)))?;
        
        match data {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| DomainError::cache(format!("Failed to deserialize products: {}", e))),
            None => Ok(Vec::new()),
        }
    }

    async fn cache_products(&self, key: &str, products: &[Product]) -> Result<(), DomainError> {
        let mut conn = self.client.clone();
        let json = serde_json::to_string(products)
            .map_err(|e| DomainError::cache(format!("Failed to serialize products: {}", e)))?;
        
        conn.set_ex(key, json, 300).await // 5 minutes TTL
            .map_err(|e| DomainError::cache(format!("Failed to cache products in Redis: {}", e)))
    }

    async fn set_value(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), DomainError> {
        let mut conn = self.client.clone();
        if let Some(ttl) = ttl_secs {
            conn.set_ex(key, value, ttl as usize).await
                .map_err(|e| DomainError::cache(format!("Failed to set value in Redis: {}", e)))
        } else {
            conn.set(key, value).await
                .map_err(|e| DomainError::cache(format!("Failed to set value in Redis: {}", e)))
        }
    }

    async fn get_value(&self, key: &str) -> Result<Option<String>, DomainError> {
        let mut conn = self.client.clone();
        conn.get(key).await
            .map_err(|e| DomainError::cache(format!("Failed to get value from Redis: {}", e)))
    }
} 