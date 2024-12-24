use async_trait::async_trait;
use scraper::{Html, Selector};
use std::collections::HashMap;
use tracing::{debug, info};
use urlencoding;

use crate::{
    error::{AppError, AppResult},
    models::{
        product::{Product, ProductPrice},
        store::Store,
    },
    services::scraper::ScraperService,
    clients::zyte::ZyteClient,
};

pub struct BauhausScraper {
    base_url: String,
    client: ZyteClient,
}

impl BauhausScraper {
    pub fn new(client: ZyteClient) -> Self {
        Self {
            base_url: "https://www.bauhaus.es".to_string(),
            client,
        }
    }

    fn parse_product_card(&self, card: scraper::ElementRef) -> Option<Product> {
        debug!("Parsing product card HTML: {}", card.html());

        // Name
        let name_selector = Selector::parse("div.product-list-tile__info__line").ok()?;
        let name = card
            .select(&name_selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        debug!("Found name: {}", name);

        // Price
        let price_wrapper_selector = Selector::parse("div.product-list-tile__price-wrapper").ok()?;
        let price_selector = Selector::parse("span.price-tag__integer-digits").ok()?;
        
        let price = card
            .select(&price_wrapper_selector)
            .next()
            .and_then(|wrapper| wrapper.select(&price_selector).next())
            .and_then(|price_tag| {
                price_tag
                    .text()
                    .collect::<String>()
                    .trim()
                    .replace(',', ".")
                    .parse::<f64>()
                    .ok()
            })
            .map(|amount| ProductPrice {
                amount,
                currency: "EUR".to_string(),
            });
        debug!("Found price: {:?}", price);

        // Description
        let desc_selector = Selector::parse("div.product-list-tile__info__attributes").ok()?;
        let description = card
            .select(&desc_selector)
            .next()
            .map(|desc| desc.text().collect::<String>().trim().to_string());
        debug!("Found description: {:?}", description);

        // URL
        let url_selector = Selector::parse("a[href]").ok()?;
        let url = card
            .select(&url_selector)
            .next()?
            .value()
            .attr("href")
            .map(|s| format!("{}{}", self.base_url, s))?;
        debug!("Found URL: {}", url);

        // Image URL - from JSON-LD, matching Python implementation
        let script_selector = Selector::parse("script[type='application/ld+json']").ok()?;
        let image_url = card
            .next_siblings()
            .find_map(|sibling| scraper::ElementRef::wrap(sibling))
            .and_then(|element| {
                if element.value().name() == "script" 
                    && element.value().attr("type") == Some("application/ld+json") {
                    Some(element)
                } else {
                    None
                }
            })
            .and_then(|script| {
                script
                    .text()
                    .collect::<String>()
                    .parse::<serde_json::Value>()
                    .ok()
                    .and_then(|json| {
                        if json["@type"] == "Product" {
                            json["image"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| Some(String::new())); // Return empty string if no image found, matching Python behavior
        debug!("Found image URL: {:?}", image_url);

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "bauhaus".to_string());

        Some(Product {
            name,
            description,
            url,
            image_url,
            price,
            store: Store::Bauhaus,
            metadata: Some(metadata),
        })
    }

    async fn scrape_url(&self, url: &str) -> AppResult<String> {
        let html = self.client.get_rendered_html(url).await?;
        debug!("Received HTML content: {}", html);
        Ok(html)
    }
}

#[async_trait]
impl ScraperService for BauhausScraper {
    async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        // Placeholder implementation
        Err(AppError::BadRequest("Not implemented".into()))
    }

    async fn search_products(&self, query: &str) -> AppResult<Vec<Product>> {
        let url = format!("{}/buscar/productos?q={}&user_search=true", self.base_url, urlencoding::encode(query));
        info!("Searching Bauhaus with URL: {}", url);

        let html = self.scrape_url(&url).await?;
        let document = Html::parse_document(&html);

        debug!("Looking for products with selector: div.product-list-tile__content-wrapper");
        let product_selector = Selector::parse("div.product-list-tile__content-wrapper")
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
            
        let products: Vec<Product> = document
            .select(&product_selector)
            .inspect(|card| debug!("Found product card: {}", card.html()))
            .filter_map(|card| self.parse_product_card(card))
            .collect();

        info!("Found {} products from Bauhaus", products.len());
        Ok(products)
    }

    async fn search_store_products(&self, store: Store, query: &str) -> AppResult<Vec<Product>> {
        if store != Store::Bauhaus {
            return Err(AppError::BadRequest("Invalid store for this scraper".into()));
        }
        self.search_products(query).await
    }
}
