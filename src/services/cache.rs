use async_trait::async_trait;
use redis::aio::ConnectionManager;

use crate::error::AppResult;

#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get(&self, key: &str) -> AppResult<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> AppResult<()>;
    async fn delete(&self, key: &str) -> AppResult<()>;
    async fn get_popular_searches(&self, limit: i32) -> AppResult<Vec<String>>;
}

#[derive(Clone)]
pub struct RedisCacheService {
    client: ConnectionManager,
}

impl RedisCacheService {
    pub fn new(client: ConnectionManager) -> Self {
        Self { client }
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self.client.clone();
        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async::<_, Option<String>>(&mut conn)
            .await?;
        Ok(value)
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> AppResult<()> {
        let mut conn = self.client.clone();
        if let Some(ttl) = ttl_secs {
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl)
                .arg(value)
                .query_async::<_, ()>(&mut conn)
                .await?;
        } else {
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.client.clone();
        redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn get_popular_searches(&self, limit: i32) -> AppResult<Vec<String>> {
        let mut conn = self.client.clone();
        let searches: Vec<String> = redis::cmd("ZREVRANGE")
            .arg("popular_searches")
            .arg(0)
            .arg(limit - 1)
            .query_async::<_, Vec<String>>(&mut conn)
            .await?;
        Ok(searches)
    }
} 