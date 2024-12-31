use async_trait::async_trait;
use crate::domain::models::DomainResult;

#[async_trait]
pub trait HttpClientPort: Send + Sync {
    async fn get(&self, url: &str) -> DomainResult<String>;
    async fn get_rendered_html(&self, url: &str) -> DomainResult<String>;
} 