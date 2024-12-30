use async_trait::async_trait;
use scraper::{Html, ElementRef, Selector};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn, error};
use crate::models::{Product, ProductPrice, Store, Selectors};
use crate::scrapers::base::BaseScraper;
use crate::error::{AppError, AppResult};
use crate::clients::zyte::ZyteClient;
use crate::services::scraper::ScraperService;

const BASE_URL: &str = "https://www.leroymerlin.es";
const SEARCH_URL: &str = "https://www.leroymerlin.es/search?q=";
const BASE_IMAGE_URL: &str = "https://media.adeo.com/media/";

/// Leroy Merlin scraper implementation
#[derive(Debug)]
pub struct LeroyScraper {
    base_url: String,
    search_url: String,
    client: ZyteClient,
}

impl LeroyScraper {
    pub fn new(client: ZyteClient) -> AppResult<Self> {
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
        
        let current_price = ProductPrice {
            amount: price,
            currency: "EUR".to_string(),
        };

        // Extract original price
        let original_price = product_data
            .get("displayed_price")
            .and_then(|p| p.as_f64())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            });

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

        match Product::new(
            name.clone(),
            String::new(),
            current_price,
            original_price,
            url,
            image_url,
            Store::LeroyMerlin,
            None,
        ) {
            Ok(product) => {
                debug!("Successfully parsed product: {}", name);
                Some(product)
            }
            Err(e) => {
                warn!("Failed to create product {}: {}", name, e);
                None
            }
        }
    }
}

#[async_trait]
impl BaseScraper for LeroyScraper {
    fn get_store_name(&self) -> &str {
        "leroy"
    }

    fn get_base_url(&self) -> &str {
        &self.base_url
    }

    fn get_search_url(&self) -> &str {
        &self.search_url
    }

    fn extract_product_info(&self, _product: &ElementRef) -> Option<Product> {
        // This is no longer used as we're using JSON parsing instead
        // but we need to implement it for the trait
        None
    }

    async fn get_product_data(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>> {
        let search_url = format!("{}{}", self.get_search_url(), query);
        info!("Fetching products from URL: {}", search_url);
        
        let html = self.client.get(&search_url).await?;
        
        let raw_products = self.extract_products_from_html(&html);
        info!("Found {} raw products", raw_products.len());

        let mut products = Vec::new();
        for product_data in raw_products.iter().take(num_products) {
            if let Some(product) = self.parse_product(product_data) {
                debug!("Successfully extracted product: {}", product.name);
                products.push(product);
            } else {
                warn!("Failed to parse product data");
            }
        }
        
        info!("Successfully extracted {} products", products.len());
        Ok(products)
    }
}

#[async_trait]
impl ScraperService for LeroyScraper {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
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
        
        Err(AppError::BadRequest("Failed to extract product details".into()))
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        self.get_product_data(query, 24).await // Default to 24 products per page
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        if store != Store::LeroyMerlin {
            return Err(AppError::BadRequest("Invalid store for this scraper".into()));
        }
        self.search_products(query).await
    }
}
