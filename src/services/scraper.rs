use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    error::AppResult,
    models::Product,
    services::{
        cache::CacheService,
        queue::RabbitMQQueue,
    },
    clients::zyte::ZyteClient,
};

#[async_trait]
pub trait ScraperService: Send + Sync {
    async fn get_product_details(&self, url: &str) -> AppResult<Product>;
    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>>;
}

pub struct ScraperServiceImpl {
    zyte_client: Arc<ZyteClient>,
    cache: Arc<dyn CacheService>,
    queue: RabbitMQQueue,
}

impl ScraperServiceImpl {
    pub fn new(
        zyte_client: Arc<ZyteClient>,
        cache: Arc<dyn CacheService>,
        queue: RabbitMQQueue,
    ) -> Self {
        Self {
            zyte_client,
            cache,
            queue,
        }
    }
}

#[async_trait]
impl ScraperService for ScraperServiceImpl {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        if let Some(product) = self.cache.get_product_details(url).await? {
            return Ok(product);
        }

        let product = self.zyte_client.get_product_details(url).await?;
        self.cache.set_product_details(&product).await?;
        Ok(product)
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        if let Some(products) = self.cache.get_search_results(query).await? {
            return Ok(products);
        }

        let products = self.zyte_client.search_products(query).await?;
        self.cache.set_search_results(query, &products).await?;
        Ok(products)
    }
} 