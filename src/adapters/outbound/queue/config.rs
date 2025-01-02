use serde::Deserialize;
use std::time::Duration;
use crate::domain::config::QueueConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMQConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub vhost: String,
    pub queue_name: String,
    pub prefetch_count: Option<u16>,
    pub connection_timeout_ms: Option<u64>,
    pub retry_interval_ms: Option<u64>,
    pub max_retries: Option<u32>,
}

impl Default for RabbitMQConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5672,
            user: "guest".to_string(),
            password: "guest".to_string(),
            vhost: "/".to_string(),
            queue_name: "scraper_queue".to_string(),
            prefetch_count: Some(1),
            connection_timeout_ms: Some(5000),
            retry_interval_ms: Some(1000),
            max_retries: Some(3),
        }
    }
}

impl QueueConfig for RabbitMQConfig {
    fn connection_url(&self) -> String {
        let vhost = if self.vhost == "/" {
            "%2F".to_string()
        } else {
            self.vhost.clone()
        };
        
        format!(
            "amqp://{}:{}@{}:{}/{}",
            self.user,
            self.password,
            self.host,
            self.port,
            vhost
        )
    }
    
    fn queue_name(&self) -> String {
        self.queue_name.clone()
    }
    
    fn connection_timeout(&self) -> Duration {
        Duration::from_millis(self.connection_timeout_ms.unwrap_or(5000))
    }
    
    fn prefetch_count(&self) -> u16 {
        self.prefetch_count.unwrap_or(1)
    }
    
    fn retry_interval(&self) -> Duration {
        Duration::from_millis(self.retry_interval_ms.unwrap_or(1000))
    }
    
    fn max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }
} 