use async_trait::async_trait;
use scraper::{Html, ElementRef, Selector};
use serde_json::Value;
use tracing::{debug, info, warn};
use url::Url;

use crate::domain::models::{Product, Store, DomainResult, DomainError};
use crate::domain::ports::outbound::{ScraperPort, HttpClientPort};

const BASE_URL: &str = "https://www.leroymerlin.es";
const SEARCH_URL: &str = "https://www.leroymerlin.es/search?q=";
const BASE_IMAGE_URL: &str = "https://media.adeo.com/media/";

/// Leroy Merlin scraper implementation
pub struct LeroyScraper {
    base_url: String,
    search_url: String,
    client: Box<dyn HttpClientPort>,
}

impl LeroyScraper {
    pub fn new(client: Box<dyn HttpClientPort>) -> DomainResult<Self> {
        Ok(Self {
            base_url: BASE_URL.to_string(),
            search_url: SEARCH_URL.to_string(),
            client,
        })
    }

    fn extract_products_from_html(&self, html: &str) -> Vec<Value> {
        let document = Html::parse_document(html);
        let script_selector = Selector::parse("script.dataTms[type='application/json']")
            .expect("Invalid script selector");
        
        for script in document.select(&script_selector) {
            if let Some(content) = script.text().next() {
                if let Ok(json) = serde_json::from_str::<Vec<Value>>(content) {
                    if !json.is_empty() {
                        if let Some(products) = json[0].get("value") {
                            if let Some(product_list) = products.as_array() {
                                debug!("Found product list in JSON data");
                                return product_list.to_vec();
                            }
                        }
                    }
                }
            }
        }
        warn!("No product data found in script tags");
        Vec::new()
    }

    fn parse_product(&self, product_data: &Value) -> Option<Product> {
        let name = match product_data.get("name")?.as_str() {
            Some(n) => n.to_string(),
            None => {
                warn!("Failed to extract product name");
                return None;
            }
        };
        
        // Extract price
        let price = product_data
            .get("offer")
            .and_then(|o| o.get("price"))
            .and_then(|p| p.as_f64())
            .unwrap_or_else(|| {
                warn!("Failed to extract price for product: {}", name);
                0.0
            });

        // Extract original price
        let original_price = product_data
            .get("displayed_price")
            .and_then(|p| p.as_f64());

        // Build URLs
        let sku = match product_data.get("sku").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => {
                warn!("Failed to extract SKU for product: {}", name);
                return None;
            }
        };

        let url = match product_data.get("url").and_then(|u| u.as_str()) {
            Some(u) => format!("{}{}", self.base_url, u),
            None => {
                warn!("Failed to extract URL for product: {}", name);
                return None;
            }
        };

        let image_url = format!("{}{}/media.jpg", BASE_IMAGE_URL, sku);

        Product::new(
            name,
            String::new(), // Empty description for now
            price,
            original_price,
            url,
            image_url,
            Store::LeroyMerlin,
        ).ok()
    }
}

#[async_trait]
impl ScraperPort for LeroyScraper {
    fn get_store(&self) -> Store {
        Store::LeroyMerlin
    }

    fn can_handle_url(&self, url: &str) -> bool {
        if let Ok(parsed_url) = Url::parse(url) {
            parsed_url.host_str()
                .map(|host| host.contains("leroymerlin.es"))
                .unwrap_or(false)
        } else {
            false
        }
    }

    async fn scrape_products(&self, query: &str, limit: Option<usize>) -> DomainResult<Vec<Product>> {
        let search_url = format!("{}{}", self.search_url, query);
        info!("Fetching products from URL: {}", search_url);
        
        let html = self.client.get(&search_url).await?;
        
        let raw_products = self.extract_products_from_html(&html);
        info!("Found {} raw products", raw_products.len());

        let mut products = Vec::new();
        for product_data in raw_products.iter().take(limit.unwrap_or(24)) {
            if let Some(product) = self.parse_product(product_data) {
                debug!("Successfully extracted product: {}", product.name());
                products.push(product);
            } else {
                warn!("Failed to parse product data");
            }
        }
        
        info!("Successfully extracted {} products", products.len());
        Ok(products)
    }

    async fn get_product_details(&self, url: &str) -> DomainResult<Product> {
        let html = self.client.get(url).await?;
        let document = Html::parse_document(&html);
        
        // Try to find the product data in JSON format
        let script_selector = Selector::parse("script.dataTms[type='application/json']")
            .expect("Invalid script selector");
            
        for script in document.select(&script_selector) {
            if let Some(content) = script.text().next() {
                if let Ok(json) = serde_json::from_str::<Vec<Value>>(content) {
                    if !json.is_empty() {
                        if let Some(products) = json[0].get("value") {
                            if let Some(product_list) = products.as_array() {
                                if let Some(product_data) = product_list.first() {
                                    if let Some(product) = self.parse_product(product_data) {
                                        return Ok(product);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err(DomainError::not_found("Failed to extract product details"))
    }
} 