use async_trait::async_trait;
use std::collections::HashMap;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{info, error};

use crate::{
    error::AppResult,
    models::{
        product::{Product, ProductPrice},
        Store,
    },
    services::scraper::ScraperService,
};

pub struct ZyteClient {
    client: Client,
    api_key: String,
    api_url: String,
}

impl ZyteClient {
    pub fn new() -> AppResult<Self> {
        let api_key = std::env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set");
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .build()?;

        Ok(Self { 
            client,
            api_key,
            api_url: "https://api.zyte.com/v1/extract".to_string(),
        })
    }

    pub async fn get_rendered_html(&self, url: &str) -> AppResult<String> {
        info!("Fetching page content from URL: {}", url);
        
        let payload = json!({
            "url": url,
            "browserHtml": true
        });

        let response = self.client
            .post(&self.api_url)
            .basic_auth(&self.api_key, Some(""))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Zyte API request failed with status code: {}",
                response.status()
            );
            error!("{}", error_msg);
            return Err(crate::error::AppError::BadRequest(error_msg));
        }

        let json_response = response.json::<Value>().await?;
        let rendered_html = json_response.get("browserHtml")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                let msg = "Rendered HTML is missing in Zyte API response.";
                error!("{}", msg);
                crate::error::AppError::BadRequest(msg.into())
            })?;

        Ok(rendered_html.to_string())
    }

    pub async fn get_api_response(&self, url: &str, options: Option<HashMap<String, Value>>) -> AppResult<Value> {
        let mut payload = json!({
            "url": url
        });

        if let Some(opts) = options {
            if let Some(obj) = payload.as_object_mut() {
                for (k, v) in opts {
                    obj.insert(k, v);
                }
            }
        }

        let response = self.client
            .post(&self.api_url)
            .basic_auth(&self.api_key, Some(""))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Zyte API request failed with status code: {}",
                response.status()
            );
            error!("{}", error_msg);
            return Err(crate::error::AppError::BadRequest(error_msg));
        }

        Ok(response.json().await?)
    }

    pub async fn scrape_product(&self, url: &str) -> AppResult<Product> {
        let mut options = HashMap::new();
        options.insert("browserHtml".to_string(), json!(true));
        options.insert("javascript".to_string(), json!(true));

        let response = self.get_api_response(url, Some(options)).await?;

        // Extract product details from response
        let product_data = response.get("product").ok_or_else(|| {
            crate::error::AppError::BadRequest("Failed to extract product data".into())
        })?;

        // Convert price data to ProductPrice
        let price = product_data.get("offers")
            .and_then(|v| v.get("price"))
            .and_then(|v| v.as_f64())
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            });

        // Convert metadata to HashMap
        let mut metadata = HashMap::new();
        if let Some(obj) = product_data.as_object() {
            for (key, value) in obj {
                if let Some(str_value) = value.as_str() {
                    metadata.insert(key.clone(), str_value.to_string());
                } else {
                    metadata.insert(key.clone(), value.to_string());
                }
            }
        }

        Ok(Product {
            name: product_data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            description: product_data.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            url: url.to_string(),
            image_url: product_data.get("image")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            price,
            store: Store::LeroyMerlin,
            metadata: Some(metadata),
        })
    }

    pub async fn scrape_search(&self, url: &str) -> AppResult<Value> {
        let html = self.get_rendered_html(url).await?;
        Ok(json!({ "browserHtml": html }))
    }
}

#[async_trait]
impl ScraperService for ZyteClient {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        self.scrape_product(url).await
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        // Construct search URL (you'll need to implement this based on your requirements)
        let search_url = format!("https://www.leroymerlin.es/search?q={}", query);
        let search_results = self.scrape_search(&search_url).await?;
        
        // Parse the search results into products
        // This is a placeholder - you'll need to implement the actual parsing logic
        let mut products = Vec::new();
        if let Some(items) = search_results.get("items").and_then(|v| v.as_array()) {
            for item in items {
                if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                    if let Ok(product) = self.get_product_details(url).await {
                        products.push(product);
                    }
                }
            }
        }
        
        Ok(products)
    }
}
