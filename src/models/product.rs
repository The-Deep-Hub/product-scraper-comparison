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
    pub description: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub price: Option<ProductPrice>,
    pub store: Store,
    pub metadata: Option<HashMap<String, String>>,
} 