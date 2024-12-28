use async_trait::async_trait;
use crate::{
    error::AppResult,
    models::{product::Product, store::Store},
    scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper},
    services::{cache::CacheService, queue::QueueService},
};

#[async_trait]
pub trait ScraperService: Send + Sync {
    fn get_store(&self) -> Store;
    async fn get_product_details(&self, url: &str) -> AppResult<Product>;
    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>>;
    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>>;
}

pub struct CombinedScraperService {
    leroy_scraper: LeroyScraper,
    bauhaus_scraper: BauhausScraper,
    bricodepot_scraper: BricodepotScraper,
    cache_service: Box<dyn CacheService>,
    queue_service: Box<dyn QueueService>,
}

impl CombinedScraperService {

/// Creates a new CombinedScraperService with the following scrapers in order:
/// 1. Leroy Merlin scraper
/// 2. Bauhaus scraper
/// 3. Bricodepot scraper

    pub fn new(
        leroy_scraper: LeroyScraper,
        bauhaus_scraper: BauhausScraper,
        bricodepot_scraper: BricodepotScraper,
        cache_service: Box<dyn CacheService>,
        queue_service: Box<dyn QueueService>,
    ) -> Self {
        Self {
            leroy_scraper,
            bauhaus_scraper,
            bricodepot_scraper,
            cache_service,
            queue_service,
        }
    }

    async fn get_store_scraper(&self, store: Store) -> AppResult<&dyn ScraperService> {
        match store {
            Store::LeroyMerlin => Ok(&self.leroy_scraper),
            Store::Bauhaus => Ok(&self.bauhaus_scraper),
            Store::Bricodepot => Ok(&self.bricodepot_scraper),
        }
    }
}

#[async_trait]
impl ScraperService for CombinedScraperService {
    fn get_store(&self) -> Store {
        // This implementation does not make sense for CombinedScraperService, 
        // as it combines multiple stores. You might want to reconsider this implementation.
        Store::LeroyMerlin
    }

    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        // Try to get from cache first
        if let Ok(Some(product)) = self.cache_service.get_product_details(url).await {
            return Ok(product);
        }

        // Determine store from URL and get appropriate scraper
        let store = Store::from_url(url)?;
        let scraper = self.get_store_scraper(store).await?;
        
        // Get product details
        let product = scraper.get_product_details(url).await?;
        
        // Cache the result
        self.cache_service.set_product_details(&product).await?;
        
        Ok(product)
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        // Try to get from cache first
        if let Ok(Some(products)) = self.cache_service.get_search_results(query).await {
            return Ok(products);
        }

        // Create a task in the queue
        let task_id = self.queue_service.create_task(query.to_string()).await?;

        // Get results from all stores
        let mut all_products = Vec::new();
        
        for store in [Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot] {
            match self.search_store_products(store, query).await {
                Ok(products) => all_products.extend(products),
                Err(e) => log::error!("Error searching store {:?}: {}", store, e),
            }
        }

        // Cache the results
        if !all_products.is_empty() {
            self.cache_service.set_search_results(query, &all_products).await?;
        }

        // Mark task as completed
        self.queue_service.mark_task_completed(task_id).await?;

        Ok(all_products)
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        // Try to get from cache first
        let cache_key = format!("{}:{}", store, query);
        if let Ok(Some(products)) = self.cache_service.get_search_results(&cache_key).await {
            return Ok(products);
        }

        // Get appropriate scraper
        let scraper = self.get_store_scraper(store).await?;
        
        // Get products
        let products = scraper.search_products(query).await?;
        
        // Cache the results
        if !products.is_empty() {
            self.cache_service.set_search_results(&cache_key, &products).await?;
        }
        
        Ok(products)
    }
} 