use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::{info, error};

use crate::domain::{
    models::{Product, DomainError, Store},
    ports::outbound::CachePort,
};

#[derive(Clone)]
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

    fn get_store_key(&self, query: &str, store: &Store) -> String {
        format!("products:{}:{}", query, store.to_string().to_lowercase())
    }

    fn get_all_stores_key(&self, query: &str) -> String {
        format!("products:{}:all", query)
    }

    async fn invalidate_store_key(&self, query: &str, store: &Store) -> Result<(), DomainError> {
        let mut conn = self.client.clone();
        let key = self.get_store_key(query, store);
        
        conn.del(&key).await
            .map_err(|e| DomainError::cache(format!("Failed to delete key {}: {}", key, e)))?;
        
        Ok(())
    }

    async fn invalidate_all_keys(&self, query: &str) -> Result<(), DomainError> {
        let mut conn = self.client.clone();
        let pattern = format!("products:{}:*", query);
        
        let keys: Vec<String> = conn.keys(&pattern).await
            .map_err(|e| DomainError::cache(format!("Failed to get keys for pattern {}: {}", pattern, e)))?;
        
        for key in keys {
            conn.del(&key).await
                .map_err(|e| DomainError::cache(format!("Failed to delete key {}: {}", key, e)))?;
        }
        Ok(())
    }
}

#[async_trait]
impl CachePort for RedisAdapter {
    async fn get_products(&self, query: &str) -> Result<Vec<Product>, DomainError> {
        let key = self.get_all_stores_key(query);
        let mut conn = self.client.clone();
        let data: Option<String> = conn.get(&key).await
            .map_err(|e| DomainError::cache(format!("Failed to get products from Redis: {}", e)))?;
        
        match data {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| DomainError::cache(format!("Failed to deserialize products: {}", e))),
            None => Ok(Vec::new()),
        }
    }

    async fn get_store_products(&self, query: &str, store: &Store) -> Result<Vec<Product>, DomainError> {
        let key = self.get_store_key(query, store);
        let mut conn = self.client.clone();
        let data: Option<String> = conn.get(&key).await
            .map_err(|e| DomainError::cache(format!("Failed to get store products from Redis: {}", e)))?;
        
        match data {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| DomainError::cache(format!("Failed to deserialize products: {}", e))),
            None => Ok(Vec::new()),
        }
    }

    async fn cache_products(&self, query: &str, products: &[Product]) -> Result<(), DomainError> {
        let mut conn = self.client.clone();
        let key = self.get_all_stores_key(query);
        let json = serde_json::to_string(products)
            .map_err(|e| DomainError::cache(format!("Failed to serialize products: {}", e)))?;
        
        conn.set_ex(&key, json, 300).await // 5 minutes TTL
            .map_err(|e| DomainError::cache(format!("Failed to cache products in Redis: {}", e)))?;

        // Also cache store-specific results without invalidating existing ones
        let store_products: std::collections::HashMap<Store, Vec<Product>> = products
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, product| {
                acc.entry(product.store().clone())
                   .or_insert_with(Vec::new)
                   .push(product.clone());
                acc
            });

        for (store, store_products) in store_products {
            let store_key = self.get_store_key(query, &store);
            let json = serde_json::to_string(&store_products)
                .map_err(|e| DomainError::cache(format!("Failed to serialize store products: {}", e)))?;
            
            conn.set_ex(&store_key, json, 300).await
                .map_err(|e| DomainError::cache(format!("Failed to cache store products in Redis: {}", e)))?;
        }

        Ok(())
    }

    async fn cache_store_products(&self, query: &str, store: &Store, products: &[Product]) -> Result<(), DomainError> {
        let mut conn = self.client.clone();
        
        // Only invalidate this store's cache
        self.invalidate_store_key(query, store).await?;
        
        // Cache store-specific products
        let key = self.get_store_key(query, store);
        let json = serde_json::to_string(products)
            .map_err(|e| DomainError::cache(format!("Failed to serialize products: {}", e)))?;
        
        conn.set_ex(&key, json, 300).await
            .map_err(|e| DomainError::cache(format!("Failed to cache store products in Redis: {}", e)))?;
        
        // Update the all-stores cache by merging with existing products
        let all_key = self.get_all_stores_key(query);
        let existing_products: Vec<Product> = match conn.get::<_, Option<String>>(&all_key).await
            .map_err(|e| DomainError::cache(format!("Failed to get existing products from Redis: {}", e)))? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| DomainError::cache(format!("Failed to deserialize existing products: {}", e)))?,
            None => Vec::new(),
        };
        
        // Remove old products for this store and add new ones
        let mut updated_products: Vec<Product> = existing_products
            .into_iter()
            .filter(|p| p.store() != *store)
            .collect();
        updated_products.extend(products.iter().cloned());
        
        // Cache updated all-stores products
        let json = serde_json::to_string(&updated_products)
            .map_err(|e| DomainError::cache(format!("Failed to serialize updated products: {}", e)))?;
        
        conn.set_ex(&all_key, json, 300).await
            .map_err(|e| DomainError::cache(format!("Failed to cache updated products in Redis: {}", e)))?;
        
        Ok(())
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