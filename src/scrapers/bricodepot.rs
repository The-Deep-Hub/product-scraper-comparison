use async_trait::async_trait;
use scraper::{Html, Selector};
use std::collections::HashMap;
use tracing::{info, error};

use crate::{
    error::{AppError, AppResult},
    models::{
        product::{Product, ProductPrice},
        store::Store,
    },
    services::scraper::ScraperService,
    clients::zyte::ZyteClient,
};

pub struct BricodepotScraper {
    base_url: String,
    search_url: String,
    client: ZyteClient,
}

impl BricodepotScraper {
    pub fn new(client: ZyteClient) -> Self {
        Self {
            base_url: "https://www.bricodepot.es".to_string(),
            search_url: "https://www.bricodepot.es/catalogsearch/result/?q=".to_string(),
            client,
        }
    }

    fn extract_products_from_html(&self, html: &str) -> Vec<Product> {
        let document = Html::parse_document(html);
        let product_selector = Selector::parse("a.product-item")
            .expect("Failed to parse product selector");

        document
            .select(&product_selector)
            .filter_map(|card| self.parse_product(card))
            .collect()
    }

    fn parse_product<'a>(&self, card: scraper::ElementRef<'a>) -> Option<Product> {
        // Extract name
        let name_selector = Selector::parse("div.product-name").ok()?;
        let name = card
            .select(&name_selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        // Extract URL
        let url = card.value().attr("href")?.to_string();

        // Extract price
        let price = self.extract_price(card);

        // Extract image URL
        let image_url = self.extract_image_url(card);

        // Extract original price for metadata
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "bricodepot".to_string());
        
        if let Some(original_price) = self.extract_original_price(card) {
            metadata.insert("original_price".to_string(), original_price.to_string());
        }

        Some(Product {
            name,
            description: None,
            url,
            image_url,
            price,
            store: Store::Bricodepot,
            metadata: Some(metadata),
        })
    }

    fn extract_price<'a>(&self, card: scraper::ElementRef<'a>) -> Option<ProductPrice> {
        let price_wrapper_selector = Selector::parse("p.price-wrapper").ok()?;
        let price_style_selector = Selector::parse("span.price-style").ok()?;
        
        let price_wrapper = card.select(&price_wrapper_selector).next()?;
        let price_style = price_wrapper.select(&price_style_selector).next()?;

        // Get integer part from first span
        let spans: Vec<_> = price_style.select(&Selector::parse("span").ok()?).collect();
        let integer_part = spans.first()?.text().collect::<String>().trim().to_string();

        // Get decimal part from span.price-top
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
        price_str.parse::<f64>().ok().map(|amount| ProductPrice {
            amount,
            currency: "EUR".to_string(),
        })
    }

    fn extract_original_price<'a>(&self, card: scraper::ElementRef<'a>) -> Option<f64> {
        let price_was_selector = Selector::parse("p.price-was").ok()?;
        let price_text = card
            .select(&price_was_selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .replace(',', ".")
            .replace('\u{a0}', "")
            .replace('€', "");

        price_text.parse::<f64>().ok()
    }

    fn extract_image_url<'a>(&self, card: scraper::ElementRef<'a>) -> Option<String> {
        let img_selector = Selector::parse("img[src]").ok()?;
        let images: Vec<_> = card
            .select(&img_selector)
            .filter_map(|img| img.value().attr("src"))
            .map(|s| s.to_string())  // Convert to owned String
            .collect();

        // Prioritize PNG images
        let png_image = images.iter().find(|src| src.ends_with(".png"));
        if let Some(src) = png_image {
            return Some(src.to_string());
        }

        // Fallback to first available image
        images.first().map(|src| src.to_string())
    }

    async fn scrape_url(&self, url: &str) -> AppResult<String> {
        let html = self.client.get_rendered_html(url).await?;
        Ok(html)
    }
}

#[async_trait]
impl ScraperService for BricodepotScraper {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        info!("Getting product details from URL: {}", url);
        
        let html = self.scrape_url(url).await?;
        let document = Html::parse_document(&html);
        
        // Find the product card on the details page
        let product_selector = Selector::parse("div.product-info-main")
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            
        if let Some(product_card) = document.select(&product_selector).next() {
            if let Some(product) = self.parse_product(product_card) {
                return Ok(product);
            }
        }
        
        Err(AppError::BadRequest("Product data not found".into()))
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        let url = format!("{}{}", self.search_url, urlencoding::encode(query));
        info!("Searching Bricodepot with URL: {}", url);

        let html = self.scrape_url(&url).await?;
        let products = self.extract_products_from_html(&html);
        
        info!("Successfully parsed {} products from Bricodepot", products.len());
        Ok(products)
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        if store != Store::Bricodepot {
            return Err(AppError::BadRequest("Invalid store for this scraper".into()));
        }
        self.search_products(query).await
    }
}
