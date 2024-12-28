use async_trait::async_trait;
use scraper::{Html, Selector};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
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
        debug!("Extracting products from HTML");
        let document = Html::parse_document(html);
        let product_selector = Selector::parse("li.product-thumbnail").expect("Failed to parse product selector");
        let mut products = Vec::new();

        for product in document.select(&product_selector) {
            let mut product_data = serde_json::Map::new();

            // Extract name
            if let Some(name_el) = product.select(&Selector::parse("span.a-designation__label").unwrap()).next() {
                let name = name_el.text().collect::<String>().trim().to_string();
                debug!("Found product: {}", name);
                product_data.insert("name".to_string(), json!(name));
            }

            // Extract current price
            if let Some(price_el) = product.select(&Selector::parse("span.js-main-price").unwrap()).next() {
                let price = price_el.text().collect::<String>().trim().to_string();
                debug!("Found price: {}", price);
                product_data.insert("js-main-price".to_string(), json!(price));
            }

            // Extract original price
            if let Some(original_price_el) = product.select(&Selector::parse("span.km-price__from-without-offer").unwrap()).next() {
                let original_price = original_price_el.text().collect::<String>().trim().to_string();
                debug!("Found original price: {}", original_price);
                product_data.insert("km-price__from-without-offer".to_string(), json!(original_price));
            }

            // Extract URL
            if let Some(url_el) = product.select(&Selector::parse("a.a-designation").unwrap()).next() {
                if let Some(href) = url_el.value().attr("href") {
                    debug!("Found URL: {}", href);
                    product_data.insert("url".to_string(), json!(href));
                }
            }

            // Extract image URL
            if let Some(img_el) = product.select(&Selector::parse("img.a-illustration__img").unwrap()).next() {
                if let Some(src) = img_el.value().attr("src") {
                    debug!("Found image URL: {}", src);
                    product_data.insert("image".to_string(), json!(src));
                }
            }

            // Extract SKU from data-reflm attribute
            if let Some(sku) = product.value().attr("data-reflm") {
                debug!("Found SKU: {}", sku);
                product_data.insert("sku".to_string(), json!(sku));
            }

            // Extract description from alt attribute of image
            if let Some(img_el) = product.select(&Selector::parse("img.a-illustration__img").unwrap()).next() {
                if let Some(alt) = img_el.value().attr("alt") {
                    debug!("Found description: {}", alt);
                    product_data.insert("description".to_string(), json!(alt));
                }
            }

            if !product_data.is_empty() {
                products.push(Value::Object(product_data));
            }
        }

        info!("Extracted {} products from HTML", products.len());
        products
    }

    fn extract_product_from_html(&self, html: &str) -> Option<Value> {
        debug!("Extracting product details from HTML");
        let document = Html::parse_document(html);
        let script_selector = Selector::parse("script[type='application/ld+json']")
            .expect("Failed to parse script selector");

        for script in document.select(&script_selector) {
            if let Some(content) = script.text().next() {
                if let Ok(data) = serde_json::from_str::<Value>(content) {
                    // Check if this is product data (should have @type: "Product")
                    if data.get("@type").and_then(|t| t.as_str()) == Some("Product") {
                        debug!("Found product JSON-LD data");
                        return Some(data);
                    }
                }
            }
        }

        warn!("No product JSON-LD data found");
        None
    }

    fn parse_product_details(&self, product_data: &Value) -> Option<Product> {
        let name = product_data.get("name")?.as_str()?.to_string();
        debug!("Parsing product details for: {}", name);
        
        // Extract current price from offers
        let current_price = product_data
            .get("offers")
            .and_then(|o| o.get("price"))
            .and_then(|p| p.as_str())
            .and_then(|p| p.parse::<f64>().ok())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            })?;

        debug!("Current price: {}", current_price.format());

        // Get original price if available
        let original_price = product_data
            .get("offers")
            .and_then(|o| o.get("originalPrice"))
            .and_then(|p| p.as_str())
            .and_then(|p| p.parse::<f64>().ok())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            });

        if let Some(ref op) = original_price {
            debug!("Original price: {}", op.format());
        }

        // Get URL (required)
        let url = product_data
            .get("url")
            .and_then(|u| u.as_str())
            .map(|u| u.to_string())?;

        debug!("Product URL: {}", url);

        // Get image URL (required)
        let image_url = product_data
            .get("image")
            .and_then(|i| i.as_str())
            .map(|i| i.to_string())?;

        debug!("Image URL: {}", image_url);

        // Get description (required, with fallback)
        let description = product_data
            .get("description")
            .and_then(|d| d.as_str())
            .map(|d| d.to_string())
            .unwrap_or_else(|| "No description available".to_string());

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "leroymerlin".to_string());
        
        // Add SKU to metadata
        if let Some(sku) = product_data.get("sku").and_then(|s| s.as_str()) {
            debug!("Found SKU: {}", sku);
            metadata.insert("sku".to_string(), sku.to_string());
        }

        Some(Product {
            name,
            description,
            current_price,
            original_price,
            url,
            image_url,
            store: Store::LeroyMerlin,
            metadata: Some(metadata),
        })
    }

    fn parse_product(&self, product_data: &Value) -> Option<Product> {
        let name = product_data.get("name")?.as_str()?.to_string();
        debug!("Parsing product: {}", name);
        
        // Extract current price from the main price element
        let current_price = product_data
            .get("js-main-price")
            .and_then(|p| p.as_str())
            .or_else(|| {
                product_data
                    .get("price")
                    .and_then(|p| p.as_str())
            })
            .and_then(|p| p.replace('€', "").trim().parse::<f64>().ok())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            })?;

        debug!("Current price: {}", current_price.format());

        // Extract original price from the crossed price
        let original_price = product_data
            .get("km-price__from-without-offer")
            .and_then(|p| p.as_str())
            .or_else(|| {
                product_data
                    .get("crossed_price")
                    .and_then(|p| p.as_str())
            })
            .and_then(|p| p.replace('€', "").trim().parse::<f64>().ok())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            });

        if let Some(ref op) = original_price {
            debug!("Original price: {}", op.format());
        }

        // Build URL
        let url = product_data
            .get("url")
            .and_then(|u| u.as_str())
            .map(|u| if u.starts_with("http") { u.to_string() } else { format!("{}{}", self.base_url, u) })?;

        debug!("Product URL: {}", url);

        // Build image URL
        let image_url = product_data
            .get("image")
            .and_then(|i| i.as_str())
            .map(|i| i.to_string())
            .or_else(|| {
                product_data
                    .get("sku")
                    .and_then(|s| s.as_str())
                    .map(|sku| format!("{}{}/media.jpg", BASE_IMAGE_URL, sku))
            })?;

        debug!("Image URL: {}", image_url);

        // Get description
        let description = product_data
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("No description available")
            .to_string();

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "leroymerlin".to_string());
        
        // Add SKU to metadata
        if let Some(sku) = product_data.get("sku").and_then(|s| s.as_str()) {
            debug!("Found SKU: {}", sku);
            metadata.insert("sku".to_string(), sku.to_string());
        }

        Some(Product {
            name,
            description,
            current_price,
            original_price,
            url,
            image_url,
            store: Store::LeroyMerlin,
            metadata: Some(metadata),
        })
    }

    async fn scrape_url(&self, url: &str) -> AppResult<String> {
        debug!("Scraping URL: {}", url);
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
                info!("Successfully extracted product details: {}", product.name);
                return Ok(product);
            }
        }
        
        warn!("Failed to extract product details from URL: {}", url);
        Err(AppError::BadRequest("Product data not found".into()))
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        info!("Searching Leroy Merlin for query: {}", query);
        let url = format!("{}/search?q={}", self.base_url, urlencoding::encode(query));
        debug!("Search URL: {}", url);

        // First get all the HTML
        let html = self.scrape_url(&url).await?;
        
        // Extract all products data
        let raw_products = self.extract_products_from_html(&html);
        info!("Found {} raw products", raw_products.len());
        
        // Now process all products
        let mut products = Vec::new();
        for product_data in raw_products {
            match self.parse_product(&product_data) {
                Some(product) => {
                    debug!("Successfully parsed product: {}", product.name);
                    products.push(product);
                }
                None => {
                    warn!("Failed to parse product from data");
                }
            }
        }

        info!("Successfully parsed {} products", products.len());
        Ok(products)
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        if store != Store::LeroyMerlin {
            return Err(AppError::BadRequest("Invalid store for this scraper".into()));
        }
        self.search_products(query).await
    }
}
