use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::error::AppResult;

pub mod zyte;

#[async_trait]
pub trait ScrapingClient: Send + Sync {
    async fn get_rendered_html(&self, url: &str) -> AppResult<String>;
    async fn get_api_response(&self, url: &str, options: Option<HashMap<String, Value>>) -> AppResult<Value>;
}
