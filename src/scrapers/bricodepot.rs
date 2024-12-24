use std::collections::HashMap;
use tracing::{error, info, debug};
use scraper::{Html, Selector, ElementRef};

use crate::{
    error::AppResult,
    models::product::{Product, ProductPrice, Store},
    clients::zyte::ZyteClient,
    config,
};

pub struct BricodepotScraper {
    client: ZyteClient,
    config: &'static config::StoreConfig,
}

impl BricodepotScraper {
    pub fn new(client: ZyteClient) -> Option<Self> {
        let config = config::get_store_config("bricodepot")?;
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
                error!("Error scraping Bricodepot: {}", e);
                Ok(vec![])
            }
        }
    }

    async fn get_product_data(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>> {
        let search_url = config::build_search_url("bricodepot", query)
            .ok_or_else(|| crate::error::AppError::BadRequest("Failed to build search URL".into()))?;
        
        info!("Fetching search results from URL: {}", search_url);
        
        let html = self.client.get_rendered_html(&search_url).await?;
        debug!("Got HTML response, length: {}", html.len());

        let document = Html::parse_document(&html);
        let product_selector = Selector::parse("a.product-item")
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
        let image_url = self.extract_image_url(card);

        if let (Some(name), Some(url), Some(price)) = (name, url, price) {
            let mut metadata = HashMap::new();
            if let Some(orig_price) = original_price {
                metadata.insert("original_price".to_string(), orig_price.to_string());
            }

            Ok(Some(Product {
                name,
                url,
                price: Some(ProductPrice {
                    amount: price,
                    currency: "EUR".to_string(),
                }),
                description: None, // Bricodepot doesn't seem to have descriptions in the product cards
                image_url,
                store: Store::Bricodepot,
                metadata: Some(metadata),
            }))
        } else {
            Ok(None)
        }
    }

    fn extract_name(&self, card: &ElementRef) -> Option<String> {
        let name_selector = Selector::parse("div.product-name").ok()?;
        card.select(&name_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    }

    fn build_full_url(&self, card: &ElementRef) -> Option<String> {
        card.value().attr("href").map(|href| {
            if href.starts_with("http") {
                href.to_string()
            } else {
                format!("{}{}", self.config.base_url, href)
            }
        })
    }

    fn extract_price(&self, card: &ElementRef) -> Option<f64> {
        let price_wrapper_selector = Selector::parse("p.price-wrapper").ok()?;
        let price_style_selector = Selector::parse("span.price-style").ok()?;
        
        let price_wrapper = card.select(&price_wrapper_selector).next()?;
        let price_style = price_wrapper.select(&price_style_selector).next()?;

        // Get integer part (first direct span child)
        let integer_part = price_style.select(&Selector::parse("span").ok()?)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        // Get decimal part (span with class price-top inside the second span)
        let decimal_selector = Selector::parse("span.price-top").ok()?;
        let decimal_part = price_style
            .select(&decimal_selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        // Combine parts and convert to float
        let price_str = format!("{}.{}", integer_part, decimal_part);
        price_str.parse::<f64>().ok()
    }

    fn extract_original_price(&self, card: &ElementRef) -> Option<f64> {
        let original_price_selector = Selector::parse("p.price-was").ok()?;
        card.select(&original_price_selector)
            .next()
            .and_then(|el| {
                el.text()
                    .collect::<String>()
                    .trim()
                    .replace(',', ".")
                    .replace('\u{a0}', "")
                    .replace('€', "")
                    .parse::<f64>()
                    .ok()
            })
    }

    fn extract_image_url(&self, card: &ElementRef) -> Option<String> {
        let img_selector = Selector::parse("img[src]").ok()?;
        let mut images: Vec<String> = card
            .select(&img_selector)
            .filter_map(|img| img.value().attr("src").map(String::from))
            .collect();

        // Prioritize PNG images
        if let Some(png_image) = images.iter().find(|url| url.ends_with(".png")) {
            Some(png_image.clone())
        } else {
            // Fallback to first available image
            images.first().cloned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bricodepot_scraper() {
        let client = ZyteClient::new().unwrap();
        let scraper = BricodepotScraper::new(client).unwrap();
        
        let products = scraper.scrape("martillo", 5).await.unwrap();
        assert!(!products.is_empty());
        
        let first_product = &products[0];
        assert!(!first_product.name.is_empty());
        assert!(first_product.url.starts_with(&scraper.config.base_url));
        assert!(first_product.price.is_some());
    }
}
