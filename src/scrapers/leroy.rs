use std::collections::HashMap;
use serde_json::Value;
use tracing::{error, info, debug};

use crate::{
    error::AppResult,
    models::product::{Product, ProductPrice, Store},
    clients::zyte::ZyteClient,
    config,
};

pub struct LeroyScraper {
    client: ZyteClient,
    config: &'static config::StoreConfig,
}

impl LeroyScraper {
    pub fn new(client: ZyteClient) -> Option<Self> {
        let config = config::get_store_config("leroy")?;
        Some(Self { client, config })
    }

    pub async fn scrape(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>> {
        info!("Starting scrape for query: {}", query);
        match self.get_product_data(query, num_products).await {
            Ok(products) => {
                info!("Successfully scraped {} products", products.len());
                Ok(products)
            }
            Err(e) => {
                error!("Error scraping Leroy Merlin: {}", e);
                Ok(vec![])
            }
        }
    }

    async fn get_product_data(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>> {
        let search_url = config::build_search_url("leroy", query)
            .ok_or_else(|| crate::error::AppError::BadRequest("Failed to build search URL".into()))?;
        
        info!("Fetching search results from URL: {}", search_url);
        
        // First, get the rendered HTML
        let html = self.client.get_rendered_html(&search_url).await?;
        debug!("Got HTML response, length: {}", html.len());

        // Extract and parse products
        let raw_products = self.extract_products_from_html(&html)?;
        debug!("Found {} raw products", raw_products.len());

        // Process only the requested number of products
        let mut products = Vec::new();
        for product_data in raw_products.iter().take(num_products) {
            if let Some(parsed_product) = self.parse_product(product_data)? {
                products.push(parsed_product);
            }
        }

        info!("Successfully parsed {} products", products.len());
        Ok(products)
    }

    fn extract_products_from_html(&self, html: &str) -> AppResult<Vec<Value>> {
        let document = scraper::Html::parse_document(html);
        
        // First, try to find the script tag with product data
        let selector = scraper::Selector::parse("script.dataTms[type='application/json']")
            .map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?;

        let mut all_products = Vec::new();
        
        // Look for product data in script tags
        for element in document.select(&selector) {
            if let Some(script_content) = element.text().next() {
                debug!("Found script content, attempting to parse");
                if let Ok(data) = serde_json::from_str::<Vec<Value>>(script_content) {
                    if !data.is_empty() {
                        if let Some(products) = data[0].get("value").and_then(|v| v.as_array()) {
                            all_products.extend(
                                products
                                    .iter()
                                    .filter(|item| {
                                        item.is_object() 
                                        && item.get("name").is_some() 
                                        && item.get("sku").is_some()
                                    })
                                    .cloned()
                            );
                        }
                    }
                }
            }
        }

        if all_products.is_empty() {
            debug!("No products found in script tags, checking alternative selectors");
            let product_selector = scraper::Selector::parse(".product-item")
                .map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?;

            for element in document.select(&product_selector) {
                if let Some(product) = self.extract_product_from_element(&element) {
                    all_products.push(product);
                }
            }
        }

        Ok(all_products)
    }

    fn extract_product_from_element(&self, element: &scraper::ElementRef) -> Option<Value> {
        let name = element.select(&scraper::Selector::parse(".product-name").ok()?).next()?.text().collect::<String>();
        let price_str = element.select(&scraper::Selector::parse(".product-price").ok()?).next()?.text().collect::<String>();
        let url = element.select(&scraper::Selector::parse("a[href]").ok()?).next()?.value().attr("href")?;
        let sku = url.split('/').last()?;

        let price = price_str
            .replace('€', "")
            .replace(',', ".")
            .trim()
            .parse::<f64>()
            .unwrap_or(0.0);

        Some(serde_json::json!({
            "name": name,
            "url": url,
            "sku": sku,
            "offer": {
                "price": price
            }
        }))
    }

    fn parse_product(&self, product_data: &Value) -> AppResult<Option<Product>> {
        match self.extract_required_fields(product_data) {
            Ok((name, url, price, _original_price, image_url)) => {
                let mut metadata = HashMap::new();
                if let Some(obj) = product_data.as_object() {
                    for (key, value) in obj {
                        metadata.insert(
                            key.clone(),
                            value.as_str()
                                .map(String::from)
                                .unwrap_or_else(|| value.to_string()),
                        );
                    }
                }

                Ok(Some(Product {
                    name,
                    url,
                    price: Some(ProductPrice {
                        amount: price,
                        currency: "EUR".to_string(),
                    }),
                    description: product_data
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    image_url: Some(image_url),
                    store: Store::LeroyMerlin,
                    metadata: Some(metadata),
                }))
            }
            Err(e) => {
                error!("Error parsing product data: {}", e);
                debug!("Problem product data: {}", serde_json::to_string_pretty(product_data)?);
                Ok(None)
            }
        }
    }

    fn extract_required_fields(&self, product_data: &Value) -> AppResult<(String, String, f64, f64, String)> {
        let name = product_data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::AppError::BadRequest("Missing product name".into()))?
            .to_string();

        let url = if let Some(url) = product_data.get("url").and_then(|v| v.as_str()) {
            if url.starts_with("http") {
                url.to_string()
            } else {
                format!("{}{}", self.config.base_url, url)
            }
        } else {
            return Err(crate::error::AppError::BadRequest("Missing product URL".into()));
        };

        let price = product_data
            .get("offer")
            .and_then(|v| v.get("price"))
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| {
                product_data
                    .get("price")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            });

        let original_price = product_data
            .get("displayed_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(price);

        let image_url = if let Some(img) = product_data.get("image").and_then(|v| v.as_str()) {
            if img.starts_with("http") {
                img.to_string()
            } else {
                format!("{}{}", self.config.base_url, img)
            }
        } else {
            let sku = product_data
                .get("sku")
                .and_then(|v| v.as_str())
                .ok_or_else(|| crate::error::AppError::BadRequest("Missing product SKU".into()))?;
            format!("{}{}/media.jpg", self.config.base_url, sku)
        };

        Ok((name, url, price, original_price, image_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_leroy_scraper() {
        let client = ZyteClient::new().unwrap();
        let scraper = LeroyScraper::new(client).unwrap();
        
        let products = scraper.scrape("martillo", 5).await.unwrap();
        assert!(!products.is_empty());
        
        let first_product = &products[0];
        assert!(!first_product.name.is_empty());
        assert!(first_product.url.starts_with(&scraper.config.base_url));
        assert!(first_product.price.is_some());
        assert!(first_product.image_url.is_some());
    }
}
