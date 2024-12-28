use async_trait::async_trait;
use scraper::{Html, ElementRef};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use crate::models::{Product, ProductPrice, Store, Selectors};
use crate::scrapers::base::BaseScraper;
use crate::error::{AppError, AppResult};
use crate::clients::zyte::ZyteClient;
use crate::services::scraper::ScraperService;

const BASE_URL: &str = "https://www.leroymerlin.es";
const SEARCH_URL: &str = "https://www.leroymerlin.es/search?q=";

/// Leroy Merlin scraper implementation
#[derive(Debug)]
pub struct LeroyScraper {
    base_url: String,
    search_url: String,
    client: ZyteClient,
    selectors: Selectors,
}

impl LeroyScraper {
    pub fn new(client: ZyteClient) -> AppResult<Self> {
        let mut selectors = HashMap::new();
        selectors.insert("product_card", "li.product-thumbnail");
        selectors.insert("title", "span.a-designation__label");
        selectors.insert("price", "span.js-main-price");
        selectors.insert("original_price", "span.km-price__from-without-offer");
        selectors.insert("url", "a.a-designation");
        selectors.insert("image", "img.a-illustration__img");
        selectors.insert("next_page", ".pagination__next-page-button");

        Ok(Self {
            base_url: BASE_URL.to_string(),
            search_url: SEARCH_URL.to_string(),
            client,
            selectors: Selectors::new(selectors)?,
        })
    }

    fn extract_price(&self, product: ElementRef) -> Option<f64> {
        let price_text = product
            .select(self.selectors.get("price")?)
            .next()?
            .text()
            .collect::<String>();

        debug!("Found price text: {}", price_text);
        
        price_text
            .trim()
            .replace('€', "")
            .replace(',', ".")
            .trim()
            .parse::<f64>()
            .ok()
    }

    fn extract_original_price(&self, product: ElementRef) -> Option<f64> {
        let price_text = product
            .select(self.selectors.get("original_price")?)
            .next()?
            .text()
            .collect::<String>();

        debug!("Found original price text: {}", price_text);

        price_text
            .trim()
            .replace('€', "")
            .replace(',', ".")
            .trim()
            .parse::<f64>()
            .ok()
    }

    fn extract_image(&self, product: ElementRef) -> Option<String> {
        product
            .select(self.selectors.get("image")?)
            .next()?
            .value()
            .attr("src")
            .map(|s| s.to_string())
    }

    fn extract_url(&self, product: ElementRef) -> Option<String> {
        product
            .select(self.selectors.get("url")?)
            .next()?
            .value()
            .attr("href")
            .map(|s| format!("{}{}", self.base_url, s))
    }

    fn extract_name(&self, product: ElementRef) -> Option<String> {
        product
            .select(self.selectors.get("title")?)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .into()
    }

    fn extract_product_info(&self, product: ElementRef) -> Option<Product> {
        let name = self.extract_name(product)?;
        let url = self.extract_url(product)?;
        let image_url = self.extract_image(product)?;
        let current_price = self.extract_price(product)?;
        let original_price = self.extract_original_price(product);

        let current_price = ProductPrice::new(current_price, "EUR".to_string()).ok()?;
        let original_price = original_price.map(|price| 
            ProductPrice::new(price, "EUR".to_string()).ok()
        ).flatten();

        Product::new(
            name,
            String::new(), // Empty description for now
            current_price,
            original_price,
            url,
            image_url,
            Store::LeroyMerlin,
            None, // No metadata for now
        ).ok()
    }

    fn has_next_page(&self, document: &Html) -> bool {
        if let Some(next_page_selector) = self.selectors.get("next_page") {
            document.select(next_page_selector).next().is_some()
        } else {
            false
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

    fn extract_product_info(&self, product: &ElementRef) -> Option<Product> {
        self.extract_product_info(*product)
    }

    async fn get_product_data(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>> {
        let search_url = format!("{}{}", self.get_search_url(), urlencoding::encode(query));
        info!("Fetching Leroy Merlin products from URL: {}", search_url);
        
        let html = self.client.get_rendered_html(&search_url).await?;
        let document = Html::parse_document(&html);
        let mut products = Vec::new();

        if let Some(product_selector) = self.selectors.get("product_card") {
            let product_cards: Vec<_> = document.select(product_selector).collect();
            info!("Found {} product cards", product_cards.len());

            for card in product_cards.iter().take(num_products) {
                if let Some(product) = self.extract_product_info(*card) {
                    debug!("Successfully extracted product: {}", product.name);
                    products.push(product);
                } else {
                    warn!("Failed to extract product info from card");
                }
            }
        }
        
        info!("Successfully extracted {} products from Leroy Merlin", products.len());
        Ok(products)
    }
}

#[async_trait]
impl ScraperService for LeroyScraper {
    fn get_store(&self) -> Store {
        Store::LeroyMerlin
    }

    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        let html = self.client.get(url).await?;
        let document = Html::parse_document(&html);
        
        if let Some(product_selector) = self.selectors.get("product_card") {
            if let Some(product) = document.select(product_selector).next() {
                if let Some(product_data) = self.extract_product_info(product) {
                    return Ok(product_data);
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
