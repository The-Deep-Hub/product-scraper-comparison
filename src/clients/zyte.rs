use async_trait::async_trait;
use reqwest::{Client, header};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tracing::{error, info};

use crate::error::{AppError, AppResult};
use super::ScrapingClient;

const API_URL: &str = "https://api.zyte.com/v1/extract";

#[derive(Debug, Clone)]
pub struct ZyteClient {
    client: Client,
    api_key: String,
    timeout: Duration,
}

impl ZyteClient {
    pub fn new() -> AppResult<Self> {
        let api_key = env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set");
        let timeout = env::var("ZYTE_REQUEST_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            api_key,
            timeout: Duration::from_secs(timeout),
        })
    }
}

#[async_trait]
impl ScrapingClient for ZyteClient {
    async fn get_rendered_html(&self, url: &str) -> AppResult<String> {
        info!("Fetching page content from URL: {}", url);

        let payload = serde_json::json!({
            "url": url,
            "browserHtml": true
        });

        let response = self.client
            .post(API_URL)
            .basic_auth(&self.api_key, Some(""))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Zyte API request failed with status code: {}", response.status());
            return Err(AppError::BadRequest(format!(
                "Zyte API request failed with status: {}",
                response.status()
            )));
        }

        let json_response = response.json::<Value>().await?;
        let rendered_html = json_response.get("browserHtml")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                error!("Rendered HTML is missing in Zyte API response");
                AppError::BadRequest("Rendered HTML is missing in Zyte API response".to_string())
            })?;

        Ok(rendered_html.to_string())
    }

    async fn get_api_response(&self, url: &str, options: Option<HashMap<String, Value>>) -> AppResult<Value> {
        let mut payload = serde_json::json!({
            "url": url
        });

        if let Some(opts) = options {
            for (key, value) in opts {
                payload[key] = value;
            }
        }

        let response = self.client
            .post(API_URL)
            .basic_auth(&self.api_key, Some(""))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Zyte API request failed with status code: {}", response.status());
            return Err(AppError::BadRequest(format!(
                "Zyte API request failed with status: {}",
                response.status()
            )));
        }

        Ok(response.json().await?)
    }
}
