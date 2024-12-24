use scraper::{Html, Selector};
use serde_json::json;
use std::sync::Arc;

use crate::clients::ScrapingClient;
use crate::error::AppResult;
use crate::models::product::{Product, ProductPrice, Store};

const BASE_URL: &str = "https://www.leroymerlin.es";
const SEARCH_URL: &str = "https://www.leroymerlin.es/buscador?term=";

#[derive(Debug, Clone)]
pub struct LeroyScraper {
    client: Arc<dyn ScrapingClient>,
}

impl LeroyScraper {
    pub fn new(client: Arc<dyn ScrapingClient>) -> Self {
        Self { client }
    }

    pub async fn search(&self, query: &str) -> AppResult<Vec<Product>> {
        let url = format!("{}{}", SEARCH_URL, query);
        let options = Some(json!({
            "browserHtml": true,
            "javascript": true,
            "actions": [
                {
                    "wait": 2000
                },
                {
                    "wait": ".product-card"
                }
            ]
        }));

        let html = self.client.get_rendered_html(&url).await?;
        let document = Html::parse_document(&html);

        let product_selector = Selector::parse(".product-card").unwrap();
        let name_selector = Selector::parse(".product-card__title").unwrap();
        let price_selector = Selector::parse(".product-card__price").unwrap();
        let link_selector = Selector::parse(".product-card__link").unwrap();
        let image_selector = Selector::parse(".product-card__image").unwrap();

        let mut products = Vec::new();

        for element in document.select(&product_selector) {
            let name = element
                .select(&name_selector)
                .next()
                .and_then(|el| el.text().next())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            let price = element
                .select(&price_selector)
                .next()
                .and_then(|el| el.text().next())
                .and_then(|s| s.trim().replace("€", "").parse::<f64>().ok())
                .unwrap_or_default();

            let url = element
                .select(&link_selector)
                .next()
                .and_then(|el| el.value().attr("href"))
                .map(|path| format!("{}{}", BASE_URL, path))
                .unwrap_or_default();

            let image_url = element
                .select(&image_selector)
                .next()
                .and_then(|el| el.value().attr("src"))
                .map(|url| url.to_string())
                .unwrap_or_default();

            let product = Product {
                name,
                description: None,
                url,
                image_url: Some(image_url),
                price: Some(ProductPrice {
                    amount: price,
                    currency: "EUR".to_string(),
                }),
                store: Store::LeroyMerlin,
                metadata: None,
            };

            products.push(product);
        }

        Ok(products)
    }

    pub async fn get_product_details(&self, url: &str) -> AppResult<Product> {
        let options = Some(json!({
            "browserHtml": true,
            "javascript": true,
            "actions": [
                {
                    "wait": 2000
                },
                {
                    "wait": ".product-detail"
                }
            ]
        }));

        let html = self.client.get_rendered_html(url).await?;
        let document = Html::parse_document(&html);

        let name_selector = Selector::parse(".product-detail__title").unwrap();
        let description_selector = Selector::parse(".product-detail__description").unwrap();
        let price_selector = Selector::parse(".product-detail__price").unwrap();
        let image_selector = Selector::parse(".product-detail__image").unwrap();

        let name = document
            .select(&name_selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let description = document
            .select(&description_selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.trim().to_string());

        let price = document
            .select(&price_selector)
            .next()
            .and_then(|el| el.text().next())
            .and_then(|s| s.trim().replace("€", "").parse::<f64>().ok())
            .unwrap_or_default();

        let image_url = document
            .select(&image_selector)
            .next()
            .and_then(|el| el.value().attr("src"))
            .map(|url| url.to_string());

        Ok(Product {
            name,
            description,
            url: url.to_string(),
            image_url,
            price: Some(ProductPrice {
                amount: price,
                currency: "EUR".to_string(),
            }),
            store: Store::LeroyMerlin,
            metadata: None,
        })
    }
}
