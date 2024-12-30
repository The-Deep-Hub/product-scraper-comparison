use async_trait::async_trait;
use scraper::{Html, ElementRef, Selector};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use urlencoding;
use crate::models::{Product, Store};
use crate::scrapers::base::BaseScraper;
use crate::error::{AppError, AppResult};
use crate::clients::zyte::ZyteClient;
use crate::services::scraper::ScraperService;
use serde_json;

pub struct BauhausScraper {
    store_name: String,
    base_url: String,
    search_url: String,
    client: ZyteClient,
    selectors: Selectors,
}

#[derive(Debug)]
struct Selectors {
    product_card: Selector,
    name: Selector,
    price_wrapper: Selector,
    price: Selector,
    original_price: Selector,
    description: Selector,
    image: Selector,
    url: Selector,
}

impl BauhausScraper {
    pub fn new(client: ZyteClient) -> Self {
        let selectors = Selectors {
            product_card: Selector::parse("div.product-list-tile").unwrap(),
            name: Selector::parse("div.product-list-tile__info__line").unwrap(),
            price_wrapper: Selector::parse("div.product-list-tile__price-wrapper").unwrap(),
            price: Selector::parse("span.price-tag__integer-digits").unwrap(),
            original_price: Selector::parse("div.price-tag__strikethrough").unwrap(),
            description: Selector::parse("div.product-list-tile__info__attributes").unwrap(),
            image: Selector::parse("img.product-list-tile__image").unwrap(),
            url: Selector::parse("a[href]").unwrap(),
        };

        Self {
            store_name: "bauhaus".to_string(),
            base_url: "https://www.bauhaus.es".to_string(),
            search_url: "https://www.bauhaus.es/buscar/productos".to_string(),
            client,
            selectors,
        }
    }

    fn extract_price(&self, card: &ElementRef) -> Option<f64> {
        let price_wrapper = card.select(&self.selectors.price_wrapper).next()?;
        let price_tag = price_wrapper.select(&self.selectors.price).next()?;
        let price_text = price_tag.text().collect::<String>();
        price_text
            .replace('€', "")
            .replace(',', ".")
            .trim()
            .parse::<f64>()
            .ok()
    }

    fn extract_original_price(&self, card: &ElementRef) -> Option<f64> {
        let price_wrapper = card.select(&self.selectors.price_wrapper).next()?;
        let strikethrough = price_wrapper.select(&self.selectors.original_price).next()?;
        
        // Try to get price from data attribute first
        if let Some(price) = strikethrough.value().attr("data-conversion-price") {
            if let Ok(amount) = price.parse::<f64>() {
                return Some(amount);
            }
        }
        
        // Fall back to text content
        let price_text = strikethrough.text().collect::<String>();
        price_text
            .replace('€', "")
            .replace(',', ".")
            .trim()
            .parse::<f64>()
            .ok()
    }
}

#[async_trait]
impl BaseScraper for BauhausScraper {
    fn get_store_name(&self) -> &str {
        &self.store_name
    }

    fn get_base_url(&self) -> &str {
        &self.base_url
    }

    fn get_search_url(&self) -> &str {
        &self.search_url
    }

    async fn get_product_data(&self, query: &str, num_products: usize) -> AppResult<Vec<Product>> {
        let search_url = format!("{}?text={}&user_search=true", self.get_search_url(), urlencoding::encode(query));
        info!("Fetching Bauhaus products from URL: {}", search_url);
        
        let html = self.fetch_search_results(&self.client, &search_url).await?;
        debug!("Received HTML content length: {}", html.len());
        
        let document = Html::parse_document(&html);
        
        // Log all unique class names to help debug selectors
        let mut unique_classes = std::collections::HashSet::new();
        for element in document.select(&Selector::parse("[class]").unwrap()) {
            if let Some(classes) = element.value().attr("class") {
                for class in classes.split_whitespace() {
                    unique_classes.insert(class.to_string());
                }
            }
        }
        debug!("Found classes on page: {:?}", unique_classes);
        
        let product_cards: Vec<_> = document.select(&self.selectors.product_card).collect();
        info!("Found {} product cards on the page", product_cards.len());
        
        let mut products = Vec::new();
        for card in product_cards.into_iter().take(num_products) {
            if let Some(product) = self.extract_product_info(&card) {
                info!("Successfully extracted product: {}", product.name);
                products.push(product);
            }
        }
        
        info!("Successfully extracted {} products from Bauhaus", products.len());
        Ok(products)
    }

    fn extract_product_info(&self, card: &ElementRef) -> Option<Product> {
        let name = card
            .select(&self.selectors.name)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string())?;
        debug!("Found product name: {}", name);

        let price = self.extract_price(card)?;
        debug!("Extracted price: {}", price);

        let original_price = self.extract_original_price(card);
        if let Some(op) = original_price {
            debug!("Extracted original price: {}", op);
        }

        // Extract image URL from JSON-LD script
        let script_selector = Selector::parse("script[type='application/ld+json']").unwrap();
        let image_url = card
            .next_siblings()
            .find_map(|sibling| {
                sibling.value().as_element().and_then(|element| {
                    if element.name() == "script" && element.attr("type") == Some("application/ld+json") {
                        let element_ref = ElementRef::wrap(sibling).unwrap();
                        let json_text = element_ref.text().collect::<String>();
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) {
                            return json.get("image").and_then(|i| i.as_str()).map(String::from);
                        }
                    }
                    None
                })
            })
            .ok_or_else(|| {
                warn!("Failed to extract image URL from JSON-LD for product {}", name);
                AppError::BadRequest("Missing image URL".into())
            })
            .ok()?;

        debug!("Extracted image URL from JSON-LD: {}", image_url);

        // Extract URL with fallback
        let url = card
            .select(&self.selectors.url)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|href| {
                if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", self.base_url, href)
                }
            })
            .ok_or_else(|| {
                warn!("Failed to extract URL from HTML for product {}", name);
                AppError::BadRequest("Missing product URL".into())
            })
            .ok()?;
        debug!("Extracted product URL: {}", url);

        // Create product with validation
        Product::new(
            name,
            String::new(), // Empty description for now
            price,
            original_price,
            url,
            image_url,
            Store::Bauhaus,
        ).ok()
    }
}

#[async_trait]
impl ScraperService for BauhausScraper {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        let html = self.fetch_search_results(&self.client, url).await?;
        let document = Html::parse_document(&html);
        
        let product_card = document
            .select(&self.selectors.product_card)
            .next()
            .ok_or_else(|| AppError::BadRequest("Product not found".into()))?;
            
        self.extract_product_info(&product_card)
            .ok_or_else(|| AppError::BadRequest("Failed to extract product info".into()))
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        self.scrape(query, Some(10)).await
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        if store != Store::Bauhaus {
            return Err(AppError::BadRequest("Invalid store for this scraper".into()));
        }
        self.search_products(query).await
    }
}
