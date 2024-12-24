use std::collections::HashMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreApi {
    pub api_type: String,
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_url: Option<String>,
    pub api: StoreApi,
}

lazy_static! {
    pub static ref STORES: HashMap<String, StoreConfig> = {
        let mut stores = HashMap::new();
        
        // Leroy Merlin Configuration
        stores.insert(
            "leroy".to_string(),
            StoreConfig {
                name: "Leroy Merlin".to_string(),
                base_url: "https://www.leroymerlin.es".to_string(),
                search_endpoint: Some("/search".to_string()),
                search_url: None,
                api: StoreApi {
                    api_type: "zyte".to_string(),
                    api_key: std::env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set"),
                    api_url: None,
                    app_id: None,
                    index_name: None,
                    headers: None,
                },
            },
        );

        // Bauhaus Configuration
        stores.insert(
            "bauhaus".to_string(),
            StoreConfig {
                name: "Bauhaus".to_string(),
                base_url: "https://www.bauhaus.es".to_string(),
                search_endpoint: Some("/buscar/productos".to_string()),
                search_url: Some("https://www.bauhaus.es/buscar/productos".to_string()),
                api: StoreApi {
                    api_type: "zyte".to_string(),
                    api_key: std::env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set"),
                    api_url: None,
                    app_id: None,
                    index_name: None,
                    headers: None,
                },
            },
        );

        // Bricodepot Configuration
        stores.insert(
            "bricodepot".to_string(),
            StoreConfig {
                name: "Bricodepot".to_string(),
                base_url: "https://www.bricodepot.es".to_string(),
                search_endpoint: None,
                search_url: Some("https://www.bricodepot.es/catalogsearch/result/".to_string()),
                api: StoreApi {
                    api_type: "algolia".to_string(),
                    api_key: "Y2I1ZDczZDMxNjkwMjZjNzNlMTcwMTdjZDZjYzdiNjgzNzE2OWZlOGMyNzAyOGJiZTNkNDNmNGUxY2M4MTM4NXRhZ0ZpbHRlcnM9JnZhbGlkVW50aWw9MTczNDg5ODM1Mw==".to_string(),
                    api_url: Some("https://jggzj7uxax-dsn.algolia.net/1/indexes/*/queries".to_string()),
                    app_id: Some("JGGZJ7UXAX".to_string()),
                    index_name: Some("pro_ES_products".to_string()),
                    headers: Some({
                        let mut headers = HashMap::new();
                        headers.insert("Content-Type".to_string(), "application/json".to_string());
                        headers
                    }),
                },
            },
        );

        // Obramat Configuration
        stores.insert(
            "obramat".to_string(),
            StoreConfig {
                name: "Obramat".to_string(),
                base_url: "https://www.obramat.es".to_string(),
                search_endpoint: None,
                search_url: Some("https://www.obramat.es/search".to_string()),
                api: StoreApi {
                    api_type: "sensefuel".to_string(),
                    api_key: "53484288-56a5-421b-a049-356b096f9840".to_string(),
                    api_url: Some("https://na.search.sensefuel.live/search/53484288-56a5-421b-a049-356b096f9840".to_string()),
                    app_id: None,
                    index_name: None,
                    headers: Some({
                        let mut headers = HashMap::new();
                        headers.insert("Content-Type".to_string(), "text/plain".to_string());
                        headers.insert("Origin".to_string(), "https://www.obramat.es".to_string());
                        headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string());
                        headers
                    }),
                },
            },
        );

        stores
    };
}

pub fn get_store_config(store: &str) -> Option<&'static StoreConfig> {
    STORES.get(store)
}

pub fn build_search_url(store: &str, query: &str) -> Option<String> {
    let config = get_store_config(store)?;
    
    match (config.search_url.as_ref(), config.search_endpoint.as_ref()) {
        (Some(search_url), _) => Some(format!("{}?q={}", search_url, query)),
        (None, Some(endpoint)) => Some(format!("{}{}?q={}", config.base_url, endpoint, query)),
        _ => None,
    }
} 