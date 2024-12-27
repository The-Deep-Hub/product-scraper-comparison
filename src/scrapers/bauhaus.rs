use async_trait::async_trait;
use scraper::{Html, Selector, ElementRef};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use urlencoding;

use crate::{
    error::{AppError, AppResult},
    models::{
        product::{Product, ProductPrice},
        store::Store,
    },
    clients::zyte::ZyteClient,
    services::scraper::ScraperService,
};

use super::base::BaseScraper;

pub struct BauhausScraper {
    store_name: String,
    base_url: String,
    search_url: String,
    client: ZyteClient,
    selectors: Selectors,
}

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
        
        // Log the first part of the HTML to see what we're getting
        if html.len() > 0 {
            debug!("First 500 chars of HTML: {}", &html[..500.min(html.len())]);
            
            // Check for common error indicators
            if html.contains("error-page") || html.contains("error-message") {
                warn!("Found error indicators in HTML response");
            }
            if html.contains("captcha") || html.contains("robot") {
                warn!("Possible bot detection/captcha found");
            }
        } else {
            warn!("Received empty HTML response");
        }
        
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
        
        // If no products found, try alternative selectors
        if product_cards.is_empty() {
            warn!("No products found with primary selector, checking alternative patterns");
            let alt_selectors = [
                "div.product-tile",
                ".product-list__item",
                ".product-grid-item",
                "[data-product-tile]"
            ];
            
            for selector in alt_selectors.iter() {
                if let Ok(alt_selector) = Selector::parse(selector) {
                    let count = document.select(&alt_selector).count();
                    debug!("Alternative selector '{}' found {} elements", selector, count);
                }
            }
        }
        
        let mut products = Vec::new();
        for card in product_cards.into_iter().take(num_products) {
            match self.extract_product_info(&card) {
                Some(product) => {
                    info!("Successfully extracted product: {}", product.name);
                    products.push(product);
                }
                None => {
                    warn!("Failed to extract product info from card");
                }
            }
        }
        
        info!("Successfully extracted {} products from Bauhaus", products.len());
        Ok(products)
    }

    fn extract_product_info(&self, card: &ElementRef) -> Option<Product> {
        // Extract name
        let name = card
            .select(&self.selectors.name)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string())?;
        debug!("Extracted name: {}", name);

        // Extract current price
        let current_price = self.extract_price(card).map(|amount| {
            debug!("Extracted current price: {}", amount);
            ProductPrice {
                amount,
                currency: "EUR".to_string(),
            }
        })?;

        // Extract original price if available
        let original_price = self.extract_original_price(card).map(|amount| {
            debug!("Extracted original price: {}", amount);
            ProductPrice {
                amount,
                currency: "EUR".to_string(),
            }
        });

        // Extract description
        let description = card
            .select(&self.selectors.description)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "No description available".to_string());
        debug!("Extracted description: {}", description);

        // Get product code from the card
        let product_code = card
            .value()
            .attr("data-product-code")
            .map(|s| s.to_string())?;
        debug!("Found product code: {}", product_code);

        // Find the matching JSON-LD script
        let json_ld = {
            let scripts = Selector::parse("script[type='application/ld+json']").unwrap();
            let fragment = card.html();
            let document = Html::parse_fragment(&fragment);
            
            document
                .select(&scripts)
                .find_map(|script| {
                    let json_text = script.text().collect::<String>();
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) {
                        if let Some(sku) = json.get("sku").and_then(|s| s.as_str()) {
                            if sku == product_code {
                                return Some(json);
                            }
                        }
                    }
                    None
                })
        };

        // Extract image URL from JSON-LD
        let image_url = if let Some(ref json) = json_ld {
            json.get("image")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
                .unwrap_or_default()
        } else {
            // Fallback to img tag
            card.select(&self.selectors.image)
                .next()
                .and_then(|img| img.value().attr("src"))
                .map(ToString::to_string)
                .unwrap_or_default()
        };
        debug!("Extracted image URL: {}", image_url);

        // Build URL from JSON-LD or fallback to href
        let url = if let Some(ref json) = json_ld {
            json.get("url")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
                .unwrap_or_else(|| {
                    card.select(&self.selectors.url)
                        .next()
                        .and_then(|a| a.value().attr("href"))
                        .map(|href| {
                            if href.starts_with("http") {
                                href.to_string()
                            } else {
                                format!("{}{}", self.base_url, href)
                            }
                        })
                        .unwrap_or_default()
                })
        } else {
            card.select(&self.selectors.url)
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(|href| {
                    if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", self.base_url, href)
                    }
                })
                .unwrap_or_default()
        };
        debug!("Built full URL: {}", url);

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "bauhaus".to_string());
        metadata.insert("sku".to_string(), product_code);

        Some(Product {
            name,
            description,
            current_price,
            original_price,
            url,
            image_url,
            store: Store::Bauhaus,
            metadata: Some(metadata),
        })
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
