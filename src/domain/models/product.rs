use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use super::error::DomainError;
use super::store::Store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductId(Uuid);

impl ProductId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    current: f64,
    original: Option<f64>,
}

impl Price {
    pub fn new(current: f64, original: Option<f64>) -> Result<Self, DomainError> {
        if current < 0.0 {
            return Err(DomainError::validation("Price cannot be negative"));
        }
        if let Some(orig) = original {
            if orig < 0.0 {
                return Err(DomainError::validation("Original price cannot be negative"));
            }
        }
        Ok(Self { current, original })
    }

    pub fn current(&self) -> f64 {
        self.current
    }

    pub fn original(&self) -> Option<f64> {
        self.original
    }

    pub fn has_discount(&self) -> bool {
        self.original
            .map(|orig| orig > self.current)
            .unwrap_or(false)
    }

    pub fn discount_percentage(&self) -> Option<f64> {
        self.original.map(|orig| {
            ((orig - self.current) / orig) * 100.0
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductUrls {
    product: String,
    image: String,
    #[serde(skip)]
    _product_url: Option<Url>,
    #[serde(skip)]
    _image_url: Option<Url>,
}

impl ProductUrls {
    pub fn new(product: String, image: String) -> Result<Self, DomainError> {
        // Validate URLs
        Url::parse(&product)
            .map_err(|_| DomainError::validation("Invalid product URL"))?;
        Url::parse(&image)
            .map_err(|_| DomainError::validation("Invalid image URL"))?;

        Ok(Self {
            product,
            image,
            _product_url: None,
            _image_url: None,
        })
    }

    pub fn product(&self) -> Result<Url, DomainError> {
        Url::parse(&self.product)
            .map_err(|_| DomainError::validation("Invalid product URL"))
    }

    pub fn image(&self) -> Result<Url, DomainError> {
        Url::parse(&self.image)
            .map_err(|_| DomainError::validation("Invalid image URL"))
    }

    pub fn product_str(&self) -> &str {
        &self.product
    }

    pub fn image_str(&self) -> &str {
        &self.image
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "id")]
    id: ProductId,
    name: String,
    description: String,
    price: Price,
    urls: ProductUrls,
    #[serde(skip_serializing, default = "Store::default")]
    store: Store,
}

impl Product {
    pub fn new(
        name: String,
        description: String,
        current_price: f64,
        original_price: Option<f64>,
        product_url: String,
        image_url: String,
        store: Store,
    ) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::validation("Product name cannot be empty"));
        }

        Ok(Self {
            id: ProductId::new(),
            name,
            description,
            price: Price::new(current_price, original_price)?,
            urls: ProductUrls::new(product_url, image_url)?,
            store,
        })
    }

    pub fn id(&self) -> &ProductId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn price(&self) -> &Price {
        &self.price
    }

    pub fn urls(&self) -> &ProductUrls {
        &self.urls
    }

    pub fn store(&self) -> Store {
        self.store
    }

    pub fn is_on_sale(&self) -> bool {
        self.price.has_discount()
    }

    pub fn discount_percentage(&self) -> Option<f64> {
        self.price.discount_percentage()
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query) || 
        self.description.to_lowercase().contains(&query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation() {
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

        let product = product.unwrap();
        assert_eq!(product.name(), "Test Product");
        assert_eq!(product.price().current(), 10.99);
        assert!(!product.is_on_sale());
    }

    #[test]
    fn test_invalid_product() {
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
    }

    #[test]
    fn test_product_with_discount() {
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
    }
} 