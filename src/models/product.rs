use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::store::Store;
use crate::error::{AppError, AppResult};
use url::Url;

/// Represents a product with its details and pricing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub description: String,
    #[serde(rename = "current_price")]
    pub price: f64,
    #[serde(rename = "original_price")]
    pub original_price: Option<f64>,
    pub url: String,
    pub image_url: String,
    pub store: Store,
}

impl Product {
    /// Creates a new Product with validation.
    /// Returns an error if any of the required fields are invalid.
    pub fn new(
        name: String,
        description: String,
        price: f64,
        original_price: Option<f64>,
        url: String,
        image_url: String,
        store: Store,
    ) -> AppResult<Self> {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("Product name cannot be empty".into()));
        }
        if price < 0.0 {
            return Err(AppError::BadRequest("Price cannot be negative".into()));
        }
        if let Some(orig_price) = original_price {
            if orig_price < 0.0 {
                return Err(AppError::BadRequest("Original price cannot be negative".into()));
            }
        }
        
        // Validate URLs
        if let Err(_) = Url::parse(&url) {
            return Err(AppError::BadRequest("Invalid product URL".into()));
        }
        if let Err(_) = Url::parse(&image_url) {
            return Err(AppError::BadRequest("Invalid image URL".into()));
        }

        Ok(Self {
            name,
            description,
            price,
            original_price,
            url,
            image_url,
            store,
        })
    }

    /// Returns true if the product is on sale (has an original price higher than current price)
    pub fn is_on_sale(&self) -> bool {
        self.original_price
            .map(|orig| orig > self.price)
            .unwrap_or(false)
    }

    /// Calculates the discount percentage if the product is on sale
    pub fn discount_percentage(&self) -> Option<f64> {
        self.original_price.map(|orig| {
            ((orig - self.price) / orig) * 100.0
        })
    }

    /// Returns a formatted string of the discount percentage
    pub fn format_discount(&self) -> Option<String> {
        self.discount_percentage()
            .map(|discount| format!("{:.0}%", discount))
    }

    /// Returns true if the product matches the given search query
    /// Case-insensitive search in name and description
    pub fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        let name = self.name.to_lowercase();
        let description = self.description.to_lowercase();

        name.contains(&query) || description.contains(&query)
    }
}

impl Default for Product {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            price: 0.0,
            original_price: None,
            url: String::new(),
            image_url: String::new(),
            store: Store::LeroyMerlin, // Default store
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_validation() {
        // Valid product
        let product = Product::new(
            "Test Product".to_string(),
            "Description".to_string(),
            10.99,
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        );
        assert!(product.is_ok());

        // Empty name
        let product = Product::new(
            "".to_string(),
            "Description".to_string(),
            10.99,
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        );
        assert!(product.is_err());

        // Invalid URL
        let product = Product::new(
            "Test Product".to_string(),
            "Description".to_string(),
            10.99,
            None,
            "not-a-url".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        );
        assert!(product.is_err());
    }

    #[test]
    fn test_product_discount() {
        let product = Product::new(
            "Test Product".to_string(),
            "Description".to_string(),
            80.0,
            Some(100.0),
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        ).unwrap();

        assert!(product.is_on_sale());
        assert_eq!(product.discount_percentage().unwrap(), 20.0);
        assert_eq!(product.format_discount().unwrap(), "20%");
    }

    #[test]
    fn test_product_search() {
        let product = Product::new(
            "Test Hammer".to_string(),
            "A great tool for DIY".to_string(),
            10.99,
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
        ).unwrap();

        assert!(product.matches_query("hammer"));
        assert!(product.matches_query("HAMMER"));
        assert!(product.matches_query("diy"));
        assert!(!product.matches_query("saw"));
    }
}