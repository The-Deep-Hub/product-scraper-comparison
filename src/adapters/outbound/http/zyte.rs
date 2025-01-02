use std::{sync::Arc, time::Duration};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION}};
use serde_json::{json, Value};
use tokio::time::sleep;
use tracing::{error};

use crate::domain::models::DomainError;
use crate::domain::ports::outbound::HttpClientPort;

#[derive(Clone)]
pub struct ZyteAdapter {
    client: Arc<Client>,
    base_url: String,
}

impl ZyteAdapter {
    const MAX_RETRIES: u32 = 3;
    const INITIAL_BACKOFF_MS: u64 = 1000; // 1 second

    pub fn new(api_key: String) -> Self {
        // Create a client with default headers
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", BASE64.encode(&format!("{}:", api_key))))
                .expect("Failed to create Authorization header"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client: Arc::new(client),
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

        Err(last_error.unwrap())
    }
}

#[async_trait]
impl HttpClientPort for ZyteAdapter {
    async fn get(&self, url: &str) -> Result<String, DomainError> {
        self.make_request_with_retry(|| async {
            let response = self.client
                .post(&self.base_url)
                .json(&json!({
                    "url": url,
                    "browserHtml": true,
                }))
                .send()
                .await
                .map_err(|e| DomainError::Http(format!("Failed to send request: {}", e)))?;

            if !response.status().is_success() {
                error!("Zyte API request failed with status code: {}", response.status());
                return Err(DomainError::Http(format!(
                    "Zyte API request failed with status code: {}",
                    response.status()
                )));
            }

            let json: Value = response
                .json()
                .await
                .map_err(|e| DomainError::Http(format!("Failed to parse response: {}", e)))?;

            Ok(json["browserHtml"]
                .as_str()
                .ok_or_else(|| DomainError::Http("No HTML content in response".to_string()))?
                .to_string())
        })
        .await
    }

    async fn get_rendered_html(&self, url: &str) -> Result<String, DomainError> {
        self.get(url).await
    }
} 