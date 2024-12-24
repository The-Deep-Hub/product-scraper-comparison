use std::time::Duration;
use redis::AsyncCommands;
use crate::{
    error::{AppError, AppResult},
    models::Product,
};

#[async_trait::async_trait]
pub trait CacheService: Send + Sync {
    async fn get_search_results(&self, query: &str) -> AppResult<Option<Vec<Product>>>;
    async fn set_search_results(&self, query: &str, products: &[Product]) -> AppResult<()>;
    async fn get_product_details(&self, url: &str) -> AppResult<Option<Product>>;
    async fn set_product_details(&self, product: &Product) -> AppResult<()>;
    async fn get_string(&self, key: &str) -> AppResult<Option<String>>;
    async fn set_string(&self, key: &str, value: &str, expiry: Option<Duration>) -> AppResult<()>;
    async fn delete(&self, key: &str) -> AppResult<()>;
}

#[derive(Clone)]
pub struct RedisCacheService {
    client: redis::aio::ConnectionManager,
}

impl RedisCacheService {
    pub fn new(client: redis::aio::ConnectionManager) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl CacheService for RedisCacheService {
    async fn get_search_results(&self, query: &str) -> AppResult<Option<Vec<Product>>> {
        if let Some(data) = self.get_string(&format!("search:{}", query)).await? {
            Ok(Some(serde_json::from_str(&data)?))
        } else {
            Ok(None)
        }
    }

    async fn set_search_results(&self, query: &str, products: &[Product]) -> AppResult<()> {
        let json = serde_json::to_string(products)?;
        self.set_string(
            &format!("search:{}", query),
            &json,
            Some(Duration::from_secs(3600)),
        )
        .await
    }

    async fn get_product_details(&self, url: &str) -> AppResult<Option<Product>> {
        if let Some(data) = self.get_string(&format!("product:{}", url)).await? {
            Ok(Some(serde_json::from_str(&data)?))
        } else {
            Ok(None)
        }
    }

    async fn set_product_details(&self, product: &Product) -> AppResult<()> {
        let json = serde_json::to_string(product)?;
        self.set_string(
            &format!("product:{}", product.url),
            &json,
            Some(Duration::from_secs(86400)),
        )
        .await
    }

    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self.client.clone();
        let result: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| AppError::RedisError(e))?;
        Ok(result)
    }

    async fn set_string(&self, key: &str, value: &str, expiry: Option<Duration>) -> AppResult<()> {
        let mut conn = self.client.clone();
        let result: () = if let Some(expiry) = expiry {
            redis::pipe()
                .atomic()
                .set(key, value)
                .expire(key, expiry.as_secs() as usize)
                .query_async(&mut conn)
                .await
        } else {
            conn.set(key, value).await
        }
        .map_err(|e| AppError::RedisError(e))?;
        Ok(result)
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.client.clone();
        let _: () = conn.del(key).await.map_err(|e| AppError::RedisError(e))?;
        Ok(())
    }
} 