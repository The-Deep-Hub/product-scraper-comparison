use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{info, error};

use crate::{
    error::AppResult,
    models::product::Product,
    clients::zyte::ZyteClient,
};

#[async_trait]
pub trait BaseScraper {
    fn get_store_name(&self) -> &str;
    fn get_base_url(&self) -> &str;
    fn get_search_url(&self) -> &str;

    async fn scrape(&self, query: &str, num_products: Option<usize>) -> AppResult<Vec<Product>> {
        info!("Starting scrape for {} with query: {}", self.get_store_name(), query);
        let products = self.get_product_data(query, num_products.unwrap_or(10)).await?;
        info!("Found {} products from {}", products.len(), self.get_store_name());
        Ok(products)
    }

    async fn get_product_data(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>>;
    
    async fn fetch_search_results(&self, client: &ZyteClient, url: &str) -> AppResult<String> {
        info!("Fetching search results from URL: {}", url);
        client.get_rendered_html(url).await
    }

    fn build_search_url(&self, query: &str) -> String {
        format!("{}?q={}", self.get_search_url(), urlencoding::encode(query))
    }

    fn extract_product_info(&self, product_data: &scraper::ElementRef) -> Option<Product>;
} 