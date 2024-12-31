use std::{sync::Arc, time::Duration};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::time::sleep;
use tracing::{info, error};

use crate::domain::models::DomainError;
use crate::domain::ports::outbound::HttpClientPort;

pub struct ZyteAdapter {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
}

impl ZyteAdapter {
    const MAX_RETRIES: u32 = 3;
    const INITIAL_BACKOFF_MS: u64 = 1000; // 1 second

    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            api_key,
            base_url: "https://api.zyte.com/v1/extract".to_string(),
        }
    }

    async fn make_request_with_retry<T, F, Fut>(&self, operation: F) -> Result<T, DomainError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, DomainError>>,
    {
        let mut retries = 0;
        let mut last_error = None;

        while retries < Self::MAX_RETRIES {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if retries == Self::MAX_RETRIES - 1 {
                        return Err(e);
                    }

                    let backoff_ms = Self::INITIAL_BACKOFF_MS * 2u64.pow(retries);
                    error!(
                        "Request failed (attempt {}/{}). Retrying in {} ms. Error: {}",
                        retries + 1,
                        Self::MAX_RETRIES,
                        backoff_ms,
                        e
                    );

                    sleep(Duration::from_millis(backoff_ms)).await;
                    retries += 1;
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| DomainError::http("Max retries exceeded")))
    }
}

#[async_trait]
impl HttpClientPort for ZyteAdapter {
    async fn get(&self, url: &str) -> Result<String, DomainError> {
        self.get_rendered_html(url).await
    }

    async fn get_rendered_html(&self, url: &str) -> Result<String, DomainError> {
        info!("Fetching page content from URL: {}", url);
        
        self.make_request_with_retry(|| async {
            let payload = json!({
                "url": url,
                "browserHtml": true
            });

            let response = self.client
                .post(&self.base_url)
                .header("Authorization", format!("Basic {}", BASE64.encode(&format!("{}:", self.api_key))))
                .json(&payload)
                .send()
                .await
                .map_err(|e| DomainError::http(format!("Failed to send request: {}", e)))?;

            if !response.status().is_success() {
                let error_msg = format!(
                    "Zyte API request failed with status code: {}",
                    response.status()
                );
                error!("{}", error_msg);
                return Err(DomainError::http(error_msg));
            }

            let json_response = response.json::<Value>().await
                .map_err(|e| DomainError::http(format!("Failed to parse JSON response: {}", e)))?;
                
            let rendered_html = json_response.get("browserHtml")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    let msg = "Rendered HTML is missing in Zyte API response.";
                    error!("{}", msg);
                    DomainError::http(msg)
                })?;

            Ok(rendered_html.to_string())
        })
        .await
    }
} 