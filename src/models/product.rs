use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::store::Store;
use crate::error::{AppError, AppResult};
use url::Url;

/// Represents a product price with an amount and currency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductPrice {
    pub amount: f64,
    pub currency: String,
}

impl ProductPrice {
    /// Creates a new ProductPrice with validation.
    /// Returns an error if the amount is negative or currency is empty.
    pub fn new(amount: f64, currency: String) -> AppResult<Self> {
        if amount < 0.0 {
            return Err(AppError::BadRequest("Price amount cannot be negative".into()));
        }
        if currency.trim().is_empty() {
            return Err(AppError::BadRequest("Currency cannot be empty".into()));
        }
        Ok(Self { amount, currency })
    }

    /// Returns a formatted string representation of the price
    pub fn format(&self) -> String {
        format!("{:.2} {}", self.amount, self.currency)
    }
}

impl Default for ProductPrice {
    fn default() -> Self {
        Self {
            amount: 0.0,
            currency: "EUR".to_string(),
        }
    }
}

/// Represents a product with its details and pricing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub description: String,
    pub current_price: ProductPrice,
    pub original_price: Option<ProductPrice>,
    pub url: String,
    pub image_url: String,
    pub store: Store,
    pub metadata: Option<HashMap<String, String>>,
}

impl Product {
    /// Creates a new Product with validation.
    /// Returns an error if any of the required fields are invalid.
    pub fn new(
        name: String,
        description: String,
        current_price: ProductPrice,
        original_price: Option<ProductPrice>,
        url: String,
        image_url: String,
        store: Store,
        metadata: Option<HashMap<String, String>>,
    ) -> AppResult<Self> {
        // Validate name
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("Product name cannot be empty".into()));
        }

        // Validate URLs
        if let Err(e) = Url::parse(&url) {
            return Err(AppError::BadRequest(format!("Invalid product URL: {}", e)));
        }
        if let Err(e) = Url::parse(&image_url) {
            return Err(AppError::BadRequest(format!("Invalid image URL: {}", e)));
        }

        Ok(Self {
            name,
            description,
            current_price,
            original_price,
            url,
            image_url,
            store,
            metadata,
        })
    }

    /// Returns true if the product is on sale (has an original price higher than current price)
    pub fn is_on_sale(&self) -> bool {
        self.original_price
            .as_ref()
            .map(|op| op.amount > self.current_price.amount)
            .unwrap_or(false)
    }

    /// Calculates the discount percentage if the product is on sale
    pub fn discount_percentage(&self) -> Option<f64> {
        self.original_price.as_ref().map(|op| {
            let discount = op.amount - self.current_price.amount;
            (discount / op.amount) * 100.0
        })
    }

    /// Returns a formatted string of the discount percentage
    pub fn format_discount(&self) -> Option<String> {
        self.discount_percentage()
            .map(|p| format!("{:.1}%", p))
    }

    /// Returns the product SKU from metadata if available
    pub fn sku(&self) -> Option<&str> {
        self.metadata
            .as_ref()
            .and_then(|m| m.get("sku"))
            .map(String::as_str)
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
            current_price: ProductPrice::default(),
            original_price: None,
            url: String::new(),
            image_url: String::new(),
            store: Store::LeroyMerlin, // Default store
            metadata: Some(HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_price_validation() {
        // Valid price
        let price = ProductPrice::new(10.99, "EUR".to_string());
        assert!(price.is_ok());

        // Negative price
        let price = ProductPrice::new(-10.99, "EUR".to_string());
        assert!(price.is_err());

        // Empty currency
        let price = ProductPrice::new(10.99, "".to_string());
        assert!(price.is_err());
    }

    #[test]
    fn test_product_validation() {
        let price = ProductPrice::new(10.99, "EUR".to_string()).unwrap();
        
        // Valid product
        let product = Product::new(
            "Test Product".to_string(),
            "Description".to_string(),
            price.clone(),
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
            None,
        );
        assert!(product.is_ok());

        // Empty name
        let product = Product::new(
            "".to_string(),
            "Description".to_string(),
            price.clone(),
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
            None,
        );
        assert!(product.is_err());

        // Invalid URL
        let product = Product::new(
            "Test Product".to_string(),
            "Description".to_string(),
            price.clone(),
            None,
            "not-a-url".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
            None,
        );
        assert!(product.is_err());
    }

    #[test]
    fn test_product_discount() {
        let current_price = ProductPrice::new(80.0, "EUR".to_string()).unwrap();
        let original_price = ProductPrice::new(100.0, "EUR".to_string()).unwrap();

        let product = Product::new(
            "Test Product".to_string(),
            "Description".to_string(),
            current_price,
            Some(original_price),
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
            None,
        ).unwrap();

        assert!(product.is_on_sale());
        assert_eq!(product.discount_percentage().unwrap(), 20.0);
        assert_eq!(product.format_discount().unwrap(), "20.0%");
    }

    #[test]
    fn test_product_search() {
        let price = ProductPrice::new(10.99, "EUR".to_string()).unwrap();
        let product = Product::new(
            "Test Hammer".to_string(),
            "A great tool for DIY".to_string(),
            price,
            None,
            "https://example.com/product".to_string(),
            "https://example.com/image.jpg".to_string(),
            Store::LeroyMerlin,
            None,
        ).unwrap();

        assert!(product.matches_query("hammer"));
        assert!(product.matches_query("HAMMER"));
        assert!(product.matches_query("diy"));
        assert!(!product.matches_query("saw"));
    }
}