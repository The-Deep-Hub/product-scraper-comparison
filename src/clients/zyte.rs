use reqwest::Client;
use serde_json::{json, Value};
use tracing::{info, error};
use std::{collections::HashMap, sync::Arc};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct ZyteClient {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
}

impl ZyteClient {
    pub fn new() -> AppResult<Self> {
        let api_key = std::env::var("ZYTE_API_KEY")
            .map_err(|_| AppError::BadRequest("ZYTE_API_KEY not set".into()))?;

        Ok(Self {
            client: Arc::new(Client::new()),
            api_key,
            base_url: "https://api.zyte.com/v1/extract".to_string(),
        })
    }

    pub async fn get_rendered_html(&self, url: &str) -> AppResult<String> {
        info!("Fetching page content from URL: {}", url);
        
        let payload = json!({
            "url": url,
            "browserHtml": true
        });

        let response = self.client
            .post(&self.base_url)
            .header("Authorization", format!("Basic {}", self.api_key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Zyte API request failed with status code: {}",
                response.status()
            );
            error!("{}", error_msg);
            return Err(AppError::BadRequest(error_msg));
        }

        let json_response = response.json::<Value>().await?;
        let rendered_html = json_response.get("browserHtml")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                let msg = "Rendered HTML is missing in Zyte API response.";
                error!("{}", msg);
                AppError::BadRequest(msg.into())
            })?;

        Ok(rendered_html.to_string())
    }

    pub async fn get(&self, url: &str) -> AppResult<String> {
        self.get_rendered_html(url).await
    }

    pub async fn get_api_response(&self, url: &str, options: Option<HashMap<String, Value>>) -> AppResult<Value> {
        let mut payload = json!({
            "url": url
        });

        if let Some(opts) = options {
            if let Some(obj) = payload.as_object_mut() {
                for (k, v) in opts {
                    obj.insert(k, v);
                }
            }
        }

        let response = self.client
            .post(&self.base_url)
            .header("Authorization", format!("Basic {}", self.api_key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Zyte API request failed with status code: {}",
                response.status()
            );
            error!("{}", error_msg);
            return Err(AppError::BadRequest(error_msg));
        }

        Ok(response.json().await?)
    }

    pub async fn get_rendered_html_with_options(&self, url: &str, options: Option<HashMap<String, Value>>) -> AppResult<String> {
        info!("Fetching page content from URL: {}", url);
        
        let mut payload = json!({
            "url": url,
            "httpResponseBody": true,
            "httpResponseHeaders": true,
            "browser": {
                "render": true
            }
        });

        if let Some(opts) = options {
            if let Some(obj) = payload.as_object_mut() {
                for (k, v) in opts {
                    obj.insert(k, v);
                }
            }
        }

        let response = self.client
            .post(&self.base_url)
            .header("Authorization", format!("Basic {}", self.api_key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!(
                "Zyte API request failed with status code: {}",
                response.status()
            );
            error!("{}", error_msg);
            return Err(AppError::BadRequest(error_msg));
        }

        let json_response = response.json::<Value>().await?;
        
        // According to Zyte docs, the rendered HTML is in browser.html when using browser.render
        let rendered_html = json_response
            .get("browser")
            .and_then(|b| b.get("html"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                let msg = "Rendered HTML is missing in Zyte API response.";
                error!("{}", msg);
                AppError::BadRequest(msg.into())
            })?;

        Ok(rendered_html.to_string())
    }
}
