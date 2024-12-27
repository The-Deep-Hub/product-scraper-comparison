use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::store::Store;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductPrice {
    pub amount: f64,
    pub currency: String,
}

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