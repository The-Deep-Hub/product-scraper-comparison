use async_trait::async_trait;
use scraper::{Html, Selector};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, error};
use urlencoding;

use crate::{
    error::{AppError, AppResult},
    models::{
        product::{Product, ProductPrice},
        store::Store,
    },
    services::scraper::ScraperService,
    clients::zyte::ZyteClient,
};

const BASE_IMAGE_URL: &str = "https://media.adeo.com/media/";

pub struct LeroyScraper {
    base_url: String,
    client: ZyteClient,
}

impl LeroyScraper {
    pub fn new(client: ZyteClient) -> Self {
        Self {
            base_url: "https://www.leroymerlin.es".to_string(),
            client,
        }
    }

    fn extract_products_from_html(&self, html: &str) -> Vec<Value> {
        let document = Html::parse_document(html);
        let script_selector = Selector::parse("script.dataTms[type='application/json']")
            .expect("Failed to parse script selector");

        let mut all_products = Vec::new();
        
        for script in document.select(&script_selector) {
            if let Some(content) = script.text().next() {
                match serde_json::from_str::<Vec<Value>>(content) {
                    Ok(data) => {
                        // Check if this is the products list
                        if let Some(first_item) = data.first() {
                            if let Some(products) = first_item.get("value").and_then(|v| v.as_array()) {
                                // Filter valid product items
                                for product in products {
                                    if product.get("name").is_some() && product.get("sku").is_some() {
                                        all_products.push(product.clone());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON from script tag: {}", e);
                    }
                }
            }
        }

        all_products
    }

    fn extract_product_from_html(&self, html: &str) -> Option<Value> {
        let document = Html::parse_document(html);
        let script_selector = Selector::parse("script[type='application/ld+json']")
            .expect("Failed to parse script selector");

        for script in document.select(&script_selector) {
            if let Some(content) = script.text().next() {
                if let Ok(data) = serde_json::from_str::<Value>(content) {
                    // Check if this is product data (should have @type: "Product")
                    if data.get("@type").and_then(|t| t.as_str()) == Some("Product") {
                        return Some(data);
                    }
                }
            }
        }

        None
    }

    fn parse_product_details(&self, product_data: &Value) -> Option<Product> {
        let name = product_data.get("name")?.as_str()?.to_string();
        
        // Extract price from offers
        let price = product_data
            .get("offers")
            .and_then(|o| o.get("price"))
            .and_then(|p| p.as_str())
            .and_then(|p| p.parse::<f64>().ok())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            });

        // Get URL
        let url = product_data
            .get("url")
            .and_then(|u| u.as_str())
            .map(|u| u.to_string())?;

        // Get image URL
        let image_url = product_data
            .get("image")
            .and_then(|i| i.as_str())
            .map(|i| i.to_string());

        // Get description
        let description = product_data
            .get("description")
            .and_then(|d| d.as_str())
            .map(|d| d.to_string());

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "leroymerlin".to_string());
        
        // Add SKU to metadata
        if let Some(sku) = product_data.get("sku").and_then(|s| s.as_str()) {
            metadata.insert("sku".to_string(), sku.to_string());
        }

        Some(Product {
            name,
            description,
            url,
            image_url,
            price,
            store: Store::LeroyMerlin,
            metadata: Some(metadata),
        })
    }

    fn parse_product(&self, product_data: &Value) -> Option<Product> {
        let name = product_data.get("name")?.as_str()?.to_string();
        
        // Extract price
        let price = product_data
            .get("offer")
            .and_then(|o| o.get("price"))
            .and_then(|p| p.as_f64())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            });

        // Build URL
        let url = product_data
            .get("url")
            .and_then(|u| u.as_str())
            .map(|u| if u.starts_with("http") { u.to_string() } else { format!("{}{}", self.base_url, u) })?;

        // Build image URL
        let image_url = product_data
            .get("sku")
            .and_then(|s| s.as_str())
            .map(|sku| format!("{}{}/media.jpg", BASE_IMAGE_URL, sku));

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "leroymerlin".to_string());
        
        // Add original price to metadata if available
        if let Some(original_price) = product_data.get("displayed_price") {
            if let Some(price_str) = original_price.as_str() {
                metadata.insert("original_price".to_string(), price_str.to_string());
            }
        }

        Some(Product {
            name,
            description: None,
            url,
            image_url,
            price,
            store: Store::LeroyMerlin,
            metadata: Some(metadata),
        })
    }

    async fn scrape_url(&self, url: &str) -> AppResult<String> {
        let html = self.client.get_rendered_html(url).await?;
        Ok(html)
    }
}

#[async_trait]
impl ScraperService for LeroyScraper {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        info!("Getting product details from URL: {}", url);
        
        let html = self.scrape_url(url).await?;
        
        // Try to extract product data from LD+JSON
        if let Some(product_data) = self.extract_product_from_html(&html) {
            if let Some(product) = self.parse_product_details(&product_data) {
                return Ok(product);
            }
        }
        
        Err(AppError::BadRequest("Product data not found".into()))
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        let url = format!("{}/search?q={}", self.base_url, urlencoding::encode(query));
        info!("Searching Leroy Merlin with URL: {}", url);

        let html = self.scrape_url(&url).await?;
        let raw_products = self.extract_products_from_html(&html);
        
        info!("Found {} raw products from Leroy Merlin", raw_products.len());
        
        let products: Vec<Product> = raw_products
            .iter()
            .filter_map(|product_data| self.parse_product(product_data))
            .collect();

        info!("Successfully parsed {} products from Leroy Merlin", products.len());
        Ok(products)
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        if store != Store::LeroyMerlin {
            return Err(AppError::BadRequest("Invalid store for this scraper".into()));
        }
        self.search_products(query).await
    }
}
