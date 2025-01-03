use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::{Product};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub stores: Vec<String>,
    #[serde(default = "default_num_products")]
    pub num_products: usize,
}

fn default_num_products() -> usize {
    100
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub task_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    pub status: String,
    pub stores: Option<HashMap<String, Vec<Product>>>,
    pub pending_stores: Option<Vec<String>>,
    pub error: Option<String>,
} 