use async_trait::async_trait;
use scraper::{Html, ElementRef, Selector};
use tracing::{debug, info, warn};
use crate::models::{Product, Store};
use crate::scrapers::base::BaseScraper;
use crate::error::{AppError, AppResult};
use crate::clients::zyte::ZyteClient;
use crate::services::scraper::ScraperService;

#[derive(Debug)]
struct Selectors {
    product_card: Selector,
    name: Selector,
    price_wrapper: Selector,
    price_style: Selector,
    price_integer: Selector,
    price_decimal: Selector,
    original_price: Selector,
    image: Selector,
}

pub struct BricodepotScraper {
    store_name: String,
    base_url: String,
    search_url: String,
    client: ZyteClient,
    selectors: Selectors,
}

impl BricodepotScraper {
    pub fn new(client: ZyteClient) -> Self {
        let selectors = Selectors {
            product_card: Selector::parse("a.product-item").unwrap(),
            name: Selector::parse("div.product-name").unwrap(),
            price_wrapper: Selector::parse("p.price-wrapper").unwrap(),
            price_style: Selector::parse("span.price-style").unwrap(),
            price_integer: Selector::parse("span:first-child").unwrap(),
            price_decimal: Selector::parse("span.price-top").unwrap(),
            original_price: Selector::parse("p.price-was").unwrap(),
            image: Selector::parse("img[src]").unwrap(),
        };

        Self {
            store_name: "bricodepot".to_string(),
            base_url: "https://www.bricodepot.es".to_string(),
            search_url: "https://www.bricodepot.es/catalogsearch/result/?q=".to_string(),
            client,
            selectors,
        }
    }

    fn extract_price(&self, card: &ElementRef) -> Option<f64> {
        debug!("Extracting price from product card");
        // Locate the price container
        let price_wrapper = card.select(&self.selectors.price_wrapper).next()?;
        
        // Extract the price style container
        let price_style = price_wrapper.select(&self.selectors.price_style).next()?;
        
        // Extract integer part
        let integer_part = price_style
            .select(&self.selectors.price_integer)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "0".to_string())
            .trim()
            .to_string();
        
        // Extract decimal part
        let decimal_part = price_style
            .select(&self.selectors.price_decimal)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| "00".to_string())
            .trim()
            .to_string();
        
        // Combine parts into float
        let combined_price = format!("{}.{}", integer_part, decimal_part);
        let price = combined_price.parse::<f64>().ok();
        debug!("Extracted price: {:?}", price);
        price
    }

    fn extract_original_price(&self, card: &ElementRef) -> Option<f64> {
        debug!("Extracting original price from product card");
        let original_price_tag = card.select(&self.selectors.original_price).next()?;
        let original_price_text = original_price_tag
            .text()
            .collect::<String>()
            .replace(',', ".")
            .replace('\u{a0}', "")
            .replace('€', "")
            .trim()
            .to_string();
        
        let price = original_price_text.parse::<f64>().ok();
        debug!("Extracted original price: {:?}", price);
        price
    }

    fn extract_image_url(&self, card: &ElementRef) -> Option<String> {
        debug!("Extracting image URL from product card");
        // Find all image tags
        let image_tags: Vec<_> = card.select(&self.selectors.image).collect();
        if image_tags.is_empty() {
            warn!("No image tags found");
            return None;
        }

        // Prioritize PNG images
        for img_tag in &image_tags {
            if let Some(src) = img_tag.value().attr("src") {
                if src.ends_with(".png") {
                    debug!("Found PNG image: {}", src);
                    return Some(src.to_string());
                }
            }
        }

        // Fallback to first available image
        let url = image_tags.first()?.value().attr("src").map(|s| s.to_string());
        debug!("Using fallback image URL: {:?}", url);
        url
    }

    fn extract_name(&self, card: &ElementRef) -> Option<String> {
        let name = card
            .select(&self.selectors.name)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        debug!("Extracted name: {}", name);
        Some(name)
    }

    fn extract_url(&self, card: &ElementRef) -> Option<String> {
        let url = card.value().attr("href")?.to_string();
        debug!("Extracted URL: {}", url);
        Some(url)
    }

    fn extract_image(&self, card: &ElementRef) -> Option<String> {
        self.extract_image_url(card)
    }
}

#[async_trait]
impl BaseScraper for BricodepotScraper {
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
        info!("Searching for '{}' products on Bricodepot", query);
        let search_url = format!("{}{}", self.get_search_url(), query);
        let html = self.fetch_search_results(&self.client, &search_url).await?;
        
        let document = Html::parse_document(&html);
        let mut products = Vec::new();
        
        let product_cards: Vec<_> = document.select(&self.selectors.product_card).collect();
        info!("Found {} product cards", product_cards.len());

        for card in product_cards.iter().take(num_products) {
            if let Some(product) = self.extract_product_info(card) {
                debug!("Successfully extracted product: {}", product.name);
                products.push(product);
            } else {
                warn!("Failed to extract product info from card");
            }
        }
        
        info!("Successfully extracted {} products", products.len());
        Ok(products)
    }

    fn extract_product_info(&self, card: &ElementRef) -> Option<Product> {
        let name = self.extract_name(card)?;
        debug!("Found product name: {}", name);

        let price = self.extract_price(card)?;
        debug!("Extracted price: {}", price);

        let original_price = self.extract_original_price(card);
        if let Some(op) = original_price {
            debug!("Extracted original price: {}", op);
        }

        let image_url = self.extract_image(card)?;
        let url = self.extract_url(card)?;

        Product::new(
            name,
            String::new(), // Empty description for now
            price,
            original_price,
            url,
            image_url,
            Store::Bricodepot,
        ).ok()
    }
}

#[async_trait]
impl ScraperService for BricodepotScraper {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        info!("Fetching product details from URL: {}", url);
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
        self.get_product_data(query, 10).await
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        if store != Store::Bricodepot {
            return Err(AppError::BadRequest("Invalid store for this scraper".into()));
        }
        self.search_products(query).await
    }
}
