use async_trait::async_trait;
use scraper::{Html, ElementRef, Selector};
use tracing::{debug, info, warn};
use url::Url;

use crate::domain::models::{Product, Store, DomainResult, DomainError};
use crate::domain::ports::outbound::{ScraperPort, HttpClientPort};

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
    search_url: String,
    client: Box<dyn HttpClientPort>,
    selectors: Selectors,
}

impl BricodepotScraper {
    pub fn new(client: Box<dyn HttpClientPort>) -> Self {
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

    async fn fetch_search_results(&self, url: &str) -> DomainResult<String> {
        self.client.get_rendered_html(url).await
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
impl ScraperPort for BricodepotScraper {
    fn get_store(&self) -> Store {
        Store::Bricodepot
    }

    fn can_handle_url(&self, url: &str) -> bool {
        if let Ok(parsed_url) = Url::parse(url) {
            parsed_url.host_str()
                .map(|host| host.contains("bricodepot.es"))
                .unwrap_or(false)
        } else {
            false
        }
    }

    async fn scrape_products(&self, query: &str, limit: Option<usize>) -> DomainResult<Vec<Product>> {
        let search_url = format!("{}{}", self.search_url, query);
        info!("Fetching products from URL: {}", search_url);
        
        let html = self.fetch_search_results(&search_url).await?;
        let document = Html::parse_document(&html);
        
        let mut products = Vec::new();
        let product_cards: Vec<_> = document.select(&self.selectors.product_card).collect();
        info!("Found {} product cards", product_cards.len());

        for card in product_cards.iter().take(limit.unwrap_or(100)) {
            if let Some(product) = self.extract_product_info(card) {
                debug!("Successfully extracted product: {}", product.name());
                products.push(product);
            } else {
                warn!("Failed to extract product info from card");
            }
        }
        
        info!("Successfully extracted {} products", products.len());
        Ok(products)
    }

    async fn get_product_details(&self, url: &str) -> DomainResult<Product> {
        info!("Fetching product details from URL: {}", url);
        let html = self.fetch_search_results(url).await?;
        let document = Html::parse_document(&html);
        
        let product_card = document
            .select(&self.selectors.product_card)
            .next()
            .ok_or_else(|| DomainError::not_found("Product not found"))?;
            
        self.extract_product_info(&product_card)
            .ok_or_else(|| DomainError::validation("Failed to extract product info"))
    }
} 