use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub base_url: String,
    pub search_endpoint: String,
    pub api: ApiConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub api_type: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct StoresConfig {
    #[serde(default)]
    pub stores: HashMap<String, Config>,
}
