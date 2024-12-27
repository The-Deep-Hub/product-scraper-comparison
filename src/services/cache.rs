use std::time::Duration;
use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::{info, error};

use crate::{
    error::{AppResult, AppError},
    models::{
        product::Product,
        task::{MainTask, StoreTask, StoreResult},
    },
};

const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get_search_results(&self, query: &str) -> AppResult<Option<Vec<Product>>>;
    async fn set_search_results(&self, query: &str, products: &[Product]) -> AppResult<()>;
    async fn get_product_details(&self, url: &str) -> AppResult<Option<Product>>;
    async fn set_product_details(&self, product: &Product) -> AppResult<()>;
    
    // New methods for task management
    async fn get_main_task(&self, task_id: &str) -> AppResult<Option<MainTask>>;
    async fn set_main_task(&self, task: &MainTask) -> AppResult<()>;
    async fn get_store_task(&self, task_id: &str) -> AppResult<Option<StoreTask>>;
    async fn set_store_task(&self, task: &StoreTask) -> AppResult<()>;
    async fn set_store_result(&self, result: &StoreResult) -> AppResult<()>;
}

pub struct RedisCacheService {
    client: ConnectionManager,
}

impl RedisCacheService {
    pub async fn new(redis_url: &str) -> AppResult<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::CacheError(format!("Failed to create Redis client: {}", e)))?
            .get_tokio_connection_manager()
            .await
            .map_err(|e| AppError::CacheError(format!("Failed to connect to Redis: {}", e)))?;
        
        Ok(Self { client })
    }
    
    fn get_main_task_key(task_id: &str) -> String {
        format!("main_task:{}", task_id)
    }
    
    fn get_store_task_key(task_id: &str) -> String {
        format!("store_task:{}", task_id)
    }
    
    fn get_search_results_key(query: &str) -> String {
        format!("search:{}", query)
    }
    
    fn get_product_key(url: &str) -> String {
        format!("product:{}", url)
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get_search_results(&self, query: &str) -> AppResult<Option<Vec<Product>>> {
        let mut conn = self.client.clone();
        let key = Self::get_search_results_key(query);
        
        let json: Option<String> = conn.get(&key).await.map_err(|e| {
            AppError::CacheError(format!("Failed to get search results: {}", e))
        })?;
        
        match json {
            Some(json) => {
                let products: Vec<Product> = serde_json::from_str(&json)?;
                Ok(Some(products))
            }
            None => Ok(None),
        }
    }
    
    async fn set_search_results(&self, query: &str, products: &[Product]) -> AppResult<()> {
        let mut conn = self.client.clone();
        let key = Self::get_search_results_key(query);
        let json = serde_json::to_string(products)?;
        
        conn.set_ex(&key, json, CACHE_TTL.as_secs() as usize).await.map_err(|e| {
            AppError::CacheError(format!("Failed to set search results: {}", e))
        })?;
        
        Ok(())
    }
    
    async fn get_product_details(&self, url: &str) -> AppResult<Option<Product>> {
        let mut conn = self.client.clone();
        let key = Self::get_product_key(url);
        
        let json: Option<String> = conn.get(&key).await.map_err(|e| {
            AppError::CacheError(format!("Failed to get product details: {}", e))
        })?;
        
        match json {
            Some(json) => {
                let product: Product = serde_json::from_str(&json)?;
                Ok(Some(product))
            }
            None => Ok(None),
        }
    }
    
    async fn set_product_details(&self, product: &Product) -> AppResult<()> {
        let mut conn = self.client.clone();
        let key = Self::get_product_key(&product.url);
        let json = serde_json::to_string(product)?;
        
        conn.set_ex(&key, json, CACHE_TTL.as_secs() as usize).await.map_err(|e| {
            AppError::CacheError(format!("Failed to set product details: {}", e))
        })?;
        
        Ok(())
    }
    
    async fn get_main_task(&self, task_id: &str) -> AppResult<Option<MainTask>> {
        let mut conn = self.client.clone();
        let key = Self::get_main_task_key(task_id);
        
        let json: Option<String> = conn.get(&key).await.map_err(|e| {
            AppError::CacheError(format!("Failed to get main task: {}", e))
        })?;
        
        match json {
            Some(json) => {
                let task: MainTask = serde_json::from_str(&json)?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }
    
    async fn set_main_task(&self, task: &MainTask) -> AppResult<()> {
        let mut conn = self.client.clone();
        let key = Self::get_main_task_key(&task.id);
        let json = serde_json::to_string(task)?;
        
        conn.set_ex(&key, json, CACHE_TTL.as_secs() as usize).await.map_err(|e| {
            AppError::CacheError(format!("Failed to set main task: {}", e))
        })?;
        
        Ok(())
    }
    
    async fn get_store_task(&self, task_id: &str) -> AppResult<Option<StoreTask>> {
        let mut conn = self.client.clone();
        let key = Self::get_store_task_key(task_id);
        
        let json: Option<String> = conn.get(&key).await.map_err(|e| {
            AppError::CacheError(format!("Failed to get store task: {}", e))
        })?;
        
        match json {
            Some(json) => {
                let task: StoreTask = serde_json::from_str(&json)?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }
    
    async fn set_store_task(&self, task: &StoreTask) -> AppResult<()> {
        let mut conn = self.client.clone();
        let key = Self::get_store_task_key(&task.id);
        let json = serde_json::to_string(task)?;
        
        conn.set_ex(&key, json, CACHE_TTL.as_secs() as usize).await.map_err(|e| {
            AppError::CacheError(format!("Failed to set store task: {}", e))
        })?;
        
        Ok(())
    }
    
    async fn set_store_result(&self, result: &StoreResult) -> AppResult<()> {
        let mut conn = self.client.clone();
        
        // Get the main task
        let main_task_key = Self::get_main_task_key(&result.main_task_id);
        let main_task_json: Option<String> = conn.get(&main_task_key).await.map_err(|e| {
            AppError::CacheError(format!("Failed to get main task: {}", e))
        })?;
        
        if let Some(json) = main_task_json {
            let mut main_task: MainTask = serde_json::from_str(&json)?;
            
            // Update main task with store results
            main_task.add_store_result(result.store.clone(), result.products.clone());
            
            // Save updated main task
            let updated_json = serde_json::to_string(&main_task)?;
            conn.set_ex(&main_task_key, updated_json, CACHE_TTL.as_secs() as usize).await.map_err(|e| {
                AppError::CacheError(format!("Failed to update main task: {}", e))
            })?;
        }
        
        Ok(())
    }
} 