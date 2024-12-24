use async_trait::async_trait;
use std::sync::Arc;

use crate::clients::ScrapingClient;
use crate::error::AppResult;
use crate::models::product::Product;
use crate::scrapers::leroy::LeroyScraper;
use crate::services::cache::CacheService;
use crate::services::queue::QueueService;

#[async_trait]
pub trait ScraperService: Send + Sync {
    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>>;
    async fn get_product_details(&self, url: &str) -> AppResult<Product>;
}

pub struct ScraperServiceImpl {
    leroy_scraper: LeroyScraper,
    cache_service: Arc<dyn CacheService>,
    queue_service: Arc<dyn QueueService>,
}

impl ScraperServiceImpl {
    pub fn new(
        scraping_client: Arc<dyn ScrapingClient>,
        cache_service: Arc<dyn CacheService>,
        queue_service: Arc<dyn QueueService>,
    ) -> Self {
        Self {
            leroy_scraper: LeroyScraper::new(scraping_client),
            cache_service,
            queue_service,
        }
    }
}

#[async_trait]
impl ScraperService for ScraperServiceImpl {
    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        // Check cache first
        if let Some(products) = self.cache_service.get_search_results(query).await? {
            return Ok(products);
        }

        // If not in cache, scrape and store
        let products = self.leroy_scraper.search(query).await?;
        
        // Store in cache
        self.cache_service.set_search_results(query, &products).await?;
        
        // Queue background tasks for detailed product info
        for product in &products {
            self.queue_service
                .enqueue_product_details_task(&product.url)
                .await?;
        }

        Ok(products)
    }

    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        // Check cache first
        if let Some(product) = self.cache_service.get_product_details(url).await? {
            return Ok(product);
        }

        // If not in cache, scrape and store
        let product = self.leroy_scraper.get_product_details(url).await?;
        
        // Store in cache
        self.cache_service.set_product_details(&product).await?;

        Ok(product)
    }
} 