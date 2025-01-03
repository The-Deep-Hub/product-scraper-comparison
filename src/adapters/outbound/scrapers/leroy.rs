use async_trait::async_trait;
use scraper::{Html, Selector};
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
        
        let mut all_products = Vec::new();
        
        for script in document.select(&script_selector) {
            if let Some(content) = script.text().next() {
                debug!("Found script content: {}", content);
                if let Ok(json) = serde_json::from_str::<Vec<Value>>(content) {
                    debug!("Parsed JSON array with {} elements", json.len());
                    if !json.is_empty() {
                        if let Some(products) = json[0].get("value") {
                            debug!("Found 'value' field: {}", products);
                            if let Some(product_list) = products.as_array() {
                                debug!("Found {} products in JSON data", product_list.len());
                                all_products.extend(product_list.iter().cloned());
                            } else {
                                warn!("'value' field is not an array");
                            }
                        } else {
                            warn!("No 'value' field found in JSON");
                        }
                    } else {
                        warn!("JSON array is empty");
                    }
                } else {
                    warn!("Failed to parse script content as JSON array");
                }
            } else {
                warn!("Script tag has no content");
            }
        }
        
        info!("Found a total of {} products across all script tags", all_products.len());
        all_products
    }

    fn parse_product(&self, product_data: &Value) -> Option<Product> {
        debug!("Parsing product data: {}", serde_json::to_string(product_data).unwrap_or_default());
        
        // Extract name
        let name = match product_data.get("name")?.as_str() {
            Some(n) => n.to_string(),
            None => {
                warn!("Failed to extract product name");
                return None;
            }
        };
        debug!("Extracted name: {}", name);
        
        // Extract price
        let price = product_data
            .get("offer")
            .and_then(|o| o.get("price"))
            .and_then(|p| p.as_f64())
            .unwrap_or_else(|| {
                warn!("Failed to extract price for product: {}", name);
                0.0
            });
        debug!("Extracted price: {}", price);

        // Extract original price
        let original_price = product_data
            .get("displayed_price")
            .and_then(|p| p.as_f64());
        debug!("Extracted original price: {:?}", original_price);

        // Build URLs
        let sku = match product_data.get("sku").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => {
                warn!("Failed to extract SKU for product: {}", name);
                return None;
            }
        };
        debug!("Extracted SKU: {}", sku);

        let url = match product_data.get("url").and_then(|u| u.as_str()) {
            Some(u) => format!("{}{}", self.base_url, u),
            None => {
                warn!("Failed to extract URL for product: {}", name);
                return None;
            }
        };
        debug!("Built URL: {}", url);

        let image_url = format!("{}{}/media.jpg", BASE_IMAGE_URL, sku);
        debug!("Built image URL: {}", image_url);

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
        // Use start and rows parameters for pagination
        let search_url = format!("{}{}?start=0&rows=100", self.search_url, query);
        info!("Fetching products from URL: {}", search_url);
        
        let html = self.client.get(&search_url).await?;
        
        let raw_products = self.extract_products_from_html(&html);
        info!("Found {} raw products", raw_products.len());

        let mut products = Vec::new();
        for product_data in raw_products.iter() {
            if let Some(product) = self.parse_product(product_data) {
                debug!("Successfully extracted product: {}", product.name());
                products.push(product);
            } else {
                warn!("Failed to parse product data");
            }
        }
        
        // Apply limit after collecting all products
        if let Some(limit) = limit {
            products.truncate(limit);
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