use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::models::{Product, Store};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub store: Option<Store>,
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
    pub error: Option<String>,
} 