use std::collections::HashMap;
use serde_json::Value;
use tracing::{error, info, debug};
use scraper::{Html, Selector, ElementRef};

use crate::{
    error::AppResult,
    models::product::{Product, ProductPrice, Store},
    clients::zyte::ZyteClient,
    config,
};

pub struct BauhausScraper {
    client: ZyteClient,
    config: &'static config::StoreConfig,
}

impl BauhausScraper {
    pub fn new(client: ZyteClient) -> Option<Self> {
        let config = config::get_store_config("bauhaus")?;
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
                error!("Error scraping Bauhaus: {}", e);
                Ok(vec![])
            }
        }
    }

    async fn get_product_data(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>> {
        let search_url = config::build_search_url("bauhaus", query)
            .ok_or_else(|| crate::error::AppError::BadRequest("Failed to build search URL".into()))?;
        
        info!("Fetching search results from URL: {}", search_url);
        
        let html = self.client.get_rendered_html(&search_url).await?;
        debug!("Got HTML response, length: {}", html.len());

        let document = Html::parse_document(&html);
        let product_selector = Selector::parse("div.product-list-tile__content-wrapper")
            .map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?;

        let mut products = Vec::new();
        for element in document.select(&product_selector).take(num_products) {
            if let Some(product) = self.extract_product_info(&element)? {
                debug!("Product scraped: {:?}", product);
                products.push(product);
            }
        }

        info!("Found {} products", products.len());
        Ok(products)
    }

    fn extract_product_info(&self, card: &ElementRef) -> AppResult<Option<Product>> {
        let name = self.extract_name(card);
        let url = self.build_full_url(card);
        let price = self.extract_price(card);
        let original_price = self.extract_original_price(card);
        let description = self.extract_description(card);
        let image_url = self.extract_image_url(card);

        if let (Some(name), Some(url), Some(price)) = (name, url, price) {
            let mut metadata = HashMap::new();
            metadata.insert("original_price".to_string(), original_price.map(|p| p.to_string()).unwrap_or_default());
            metadata.insert("description".to_string(), description.as_ref().map(String::as_str).unwrap_or_default().to_string());

            Ok(Some(Product {
                name,
                url,
                price: Some(ProductPrice {
                    amount: price,
                    currency: "EUR".to_string(),
                }),
                description,
                image_url,
                store: Store::Bauhaus,
                metadata: Some(metadata),
            }))
        } else {
            Ok(None)
        }
    }

    fn extract_name(&self, card: &ElementRef) -> Option<String> {
        let name_selector = Selector::parse("div.product-list-tile__info__line").ok()?;
        card.select(&name_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    }

    fn build_full_url(&self, card: &ElementRef) -> Option<String> {
        let url_selector = Selector::parse("a[href]").ok()?;
        let href = card.select(&url_selector)
            .next()
            .and_then(|el| el.value().attr("href"))?;
            
        if href.starts_with("http") {
            Some(href.to_string())
        } else {
            Some(format!("{}{}", self.config.base_url, href))
        }
    }

    fn extract_price(&self, card: &ElementRef) -> Option<f64> {
        let price_wrapper_selector = Selector::parse("div.product-list-tile__price-wrapper").ok()?;
        let price_selector = Selector::parse("span.price-tag__integer-digits").ok()?;

        card.select(&price_wrapper_selector)
            .next()?
            .select(&price_selector)
            .next()
            .map(|el| {
                el.text()
                    .collect::<String>()
                    .trim()
                    .replace(',', ".")
                    .parse::<f64>()
                    .ok()
            })
            .flatten()
    }

    fn extract_original_price(&self, card: &ElementRef) -> Option<f64> {
        let price_wrapper_selector = Selector::parse("div.product-list-tile__price-wrapper").ok()?;
        let strikethrough_selector = Selector::parse("div.price-tag__strikethrough").ok()?;

        let price_wrapper = card.select(&price_wrapper_selector).next()?;
        let strikethrough = price_wrapper.select(&strikethrough_selector).next()?;

        // Try to get price from data attribute first
        if let Some(price_attr) = strikethrough.value().attr("data-conversion-price") {
            if let Ok(price) = price_attr.parse::<f64>() {
                return Some(price);
            }
        }

        // Fall back to strikethrough text
        let s_selector = Selector::parse("s").ok()?;
        strikethrough
            .select(&s_selector)
            .next()
            .map(|el| {
                el.text()
                    .collect::<String>()
                    .trim()
                    .replace(',', ".")
                    .parse::<f64>()
                    .ok()
            })
            .flatten()
    }

    fn extract_description(&self, card: &ElementRef) -> Option<String> {
        let desc_selector = Selector::parse("div.product-list-tile__info__attributes").ok()?;
        card.select(&desc_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    }

    fn extract_image_url(&self, card: &ElementRef) -> Option<String> {
        let jsonld_selector = Selector::parse("script[type='application/ld+json']").ok()?;
        let jsonld_tag = card.select(&jsonld_selector).next()?;
        
        let jsonld_text = jsonld_tag.text().collect::<String>();
        if let Ok(jsonld_data) = serde_json::from_str::<Value>(&jsonld_text) {
            if jsonld_data.get("@type").and_then(|t| t.as_str()) == Some("Product") {
                return jsonld_data.get("image").and_then(|i| i.as_str()).map(String::from);
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bauhaus_scraper() {
        let client = ZyteClient::new().unwrap();
        let scraper = BauhausScraper::new(client).unwrap();
        
        let products = scraper.scrape("martillo", 5).await.unwrap();
        assert!(!products.is_empty());
        
        let first_product = &products[0];
        assert!(!first_product.name.is_empty());
        assert!(first_product.url.starts_with(scraper.config.base_url.as_ref().unwrap()));
        assert!(first_product.price.is_some());
    }
}
