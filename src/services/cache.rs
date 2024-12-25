use redis::{aio::ConnectionManager, AsyncCommands, Pipeline};
use std::time::Duration;
use tracing::info;

use crate::{
    error::AppResult,
    models::{product::Product, store::Store},
};

const PRODUCT_CACHE_PREFIX: &str = "product";
const STORE_SEARCH_CACHE_PREFIX: &str = "store_search";
const GENERAL_SEARCH_CACHE_PREFIX: &str = "search";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[async_trait::async_trait]
pub trait CacheService: Send + Sync {
    async fn get_product_details(&self, url: &str) -> AppResult<Option<Product>>;
    async fn set_product_details(&self, product: &Product) -> AppResult<()>;
    async fn get_search_results(&self, query: &str) -> AppResult<Option<Vec<Product>>>;
    async fn set_search_results(&self, query: &str, products: &[Product]) -> AppResult<()>;
}

pub struct RedisCacheService {
    client: ConnectionManager,
}

impl RedisCacheService {
    pub async fn new(redis_url: &str) -> AppResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let client = ConnectionManager::new(client).await?;
        Ok(Self { client })
    }

    fn get_product_key(url: &str) -> String {
        format!("{}:{}", PRODUCT_CACHE_PREFIX, url)
    }

    fn get_store_search_key(store: &Store, query: &str) -> String {
        format!("{}:{}:{}", STORE_SEARCH_CACHE_PREFIX, store, query)
    }

    fn get_general_search_key(query: &str) -> String {
        format!("{}:{}", GENERAL_SEARCH_CACHE_PREFIX, query)
    }
}

#[async_trait::async_trait]
impl CacheService for RedisCacheService {
    async fn get_search_results(&self, query: &str) -> AppResult<Option<Vec<Product>>> {
        let mut conn = self.client.clone();
        let key = Self::get_general_search_key(query);
        let data: Option<String> = conn.get(&key).await?;
        
        if let Some(data) = data {
            info!("Cache hit for search query: {}", query);
            Ok(Some(serde_json::from_str(&data)?))
        } else {
            info!("Cache miss for search query: {}", query);
            Ok(None)
        }
    }

    async fn set_search_results(&self, query: &str, products: &[Product]) -> AppResult<()> {
        let mut conn = self.client.clone();
        let json = serde_json::to_string(products)?;
        let key = Self::get_general_search_key(query);
        
        info!("Caching search results for query: {}", query);
        conn.set_ex(&key, json, CACHE_TTL.as_secs() as usize).await?;

        // Also cache store-specific results
        let mut pipe = Pipeline::new();
        let products_by_store = products.iter().fold(std::collections::HashMap::new(), |mut acc, product| {
            acc.entry(&product.store).or_insert_with(Vec::new).push(product);
            acc
        });

        for (store, store_products) in products_by_store {
            let store_key = Self::get_store_search_key(store, query);
            let store_json = serde_json::to_string(&store_products)?;
            pipe.set_ex(&store_key, store_json, CACHE_TTL.as_secs() as usize);
        }

        pipe.query_async(&mut conn).await?;
        Ok(())
    }

    async fn get_product_details(&self, url: &str) -> AppResult<Option<Product>> {
        let mut conn = self.client.clone();
        let key = Self::get_product_key(url);
        let data: Option<String> = conn.get(&key).await?;
        
        if let Some(data) = data {
            info!("Cache hit for product URL: {}", url);
            Ok(Some(serde_json::from_str(&data)?))
        } else {
            info!("Cache miss for product URL: {}", url);
            Ok(None)
        }
    }

    async fn set_product_details(&self, product: &Product) -> AppResult<()> {
        let mut conn = self.client.clone();
        let key = Self::get_product_key(&product.url);
        let json = serde_json::to_string(product)?;
        
        info!("Caching product: {}", product.url);
        conn.set_ex(&key, json, CACHE_TTL.as_secs() as usize).await?;
        Ok(())
    }
}

pub struct BatchCacheService {
    client: ConnectionManager,
}

impl BatchCacheService {
    pub async fn new(redis_url: &str) -> AppResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let client = ConnectionManager::new(client).await?;
        Ok(Self { client })
    }

    pub async fn set_products_batch(&self, products: &[Product]) -> AppResult<()> {
        let mut conn = self.client.clone();
        let mut pipe = Pipeline::new();

        for product in products {
            let key = format!("{}:{}", PRODUCT_CACHE_PREFIX, product.url);
            let json = serde_json::to_string(product)?;
            pipe.set_ex(&key, json, CACHE_TTL.as_secs() as usize);
        }

        pipe.query_async(&mut conn).await?;
        Ok(())
    }
} 