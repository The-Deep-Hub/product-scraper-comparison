use std::sync::Arc;

use crate::domain::models::{Product, Store, DomainResult, DomainError};
use crate::domain::ports::outbound::ScraperPort;

pub struct ScraperService {
    scrapers: Vec<Arc<dyn ScraperPort>>,
}

impl ScraperService {
    pub fn new(scrapers: Vec<Arc<dyn ScraperPort>>) -> Self {
        Self { scrapers }
    }

    pub fn register_scraper(&mut self, scraper: Arc<dyn ScraperPort>) {
        self.scrapers.push(scraper);
    }

    pub async fn scrape_products(&self, query: &str, limit: Option<usize>) -> DomainResult<Vec<Product>> {
        let mut all_products = Vec::new();
        
        for scraper in &self.scrapers {
            match scraper.scrape_products(query, limit).await {
                Ok(products) => all_products.extend(products),
                Err(e) => tracing::warn!("Error scraping products: {}", e),
            }
        }

        Ok(all_products)
    }

    pub async fn scrape_store_products(&self, store: Store, query: &str, limit: Option<usize>) -> DomainResult<Vec<Product>> {
        for scraper in &self.scrapers {
            if scraper.get_store() == store {
                return scraper.scrape_products(query, limit).await;
            }
        }
        
        Ok(Vec::new())
    }

    pub async fn get_product_details(&self, url: &str) -> DomainResult<Product> {
        for scraper in &self.scrapers {
            if scraper.can_handle_url(url) {
                return scraper.get_product_details(url).await;
            }
        }
        
        Err(DomainError::not_found("No scraper found for URL"))
    }
} 