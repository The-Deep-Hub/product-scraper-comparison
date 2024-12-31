use rust_scraper::domain::models::{Product, Store, Price, ProductUrls, DomainError};

#[test]
fn test_store_operations() {
    assert_eq!(Store::LeroyMerlin.as_str(), "leroy");
    assert_eq!(Store::from_str("leroy"), Some(Store::LeroyMerlin));
    assert_eq!(Store::from_str("invalid"), None);
}

#[test]
fn test_price_operations() {
    let price = Price::new(80.0, Some(100.0)).unwrap();
    assert!(price.has_discount());
    assert_eq!(price.discount_percentage().unwrap(), 20.0);
    
    // Test invalid price
    assert!(Price::new(-10.0, None).is_err());
}

#[test]
fn test_product_urls() {
    let urls = ProductUrls::new(
        "https://example.com/product".to_string(),
        "https://example.com/image.jpg".to_string(),
    );
    assert!(urls.is_ok());
    
    let urls = urls.unwrap();
    assert!(urls.product().is_ok());
    assert!(urls.image().is_ok());
    assert_eq!(urls.product_str(), "https://example.com/product");
    assert_eq!(urls.image_str(), "https://example.com/image.jpg");
    
    // Test invalid URL
    let invalid_urls = ProductUrls::new(
        "not-a-url".to_string(),
        "https://example.com/image.jpg".to_string(),
    );
    assert!(matches!(invalid_urls, Err(DomainError::Validation(_))));
}

#[tokio::test]
async fn test_product_creation_and_validation() {
    // Test valid product creation
    let product = Product::new(
        "Test Product".to_string(),
        "Description".to_string(),
        10.99,
        Some(15.99),
        "https://example.com/product".to_string(),
        "https://example.com/image.jpg".to_string(),
        Store::LeroyMerlin,
    );
    assert!(product.is_ok());
    
    let product = product.unwrap();
    assert!(product.is_on_sale());
    assert!(product.discount_percentage().unwrap() > 0.0);
    
    // Test invalid product creation
    let invalid_product = Product::new(
        "".to_string(), // Empty name
        "Description".to_string(),
        10.99,
        None,
        "https://example.com/product".to_string(),
        "https://example.com/image.jpg".to_string(),
        Store::LeroyMerlin,
    );
    assert!(matches!(invalid_product, Err(DomainError::Validation(_))));
} 