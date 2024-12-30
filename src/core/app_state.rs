use std::sync::Arc;
use anyhow::Result;
use tracing::info;

use crate::{
    config::AppConfig,
    clients::zyte::ZyteClient,
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        scraper::{ScraperService, CombinedScraperService},
        task_splitter::{TaskSplitterService, RabbitMQTaskSplitter},
    },
    scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub cache_service: Arc<dyn CacheService>,
    pub queue_service: Arc<dyn QueueService>,
    pub scraper_service: Arc<dyn ScraperService>,
    pub task_splitter: Arc<dyn TaskSplitterService>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Load configuration
        let config = Arc::new(AppConfig::new()?);
        
        // Initialize Redis
        let redis_url = config.as_ref().redis_url();
        info!("Connecting to Redis at: {}", Self::mask_password(&redis_url));
        let cache_service = Arc::new(RedisCacheService::new(&redis_url).await?) as Arc<dyn CacheService>;
        
        // Initialize RabbitMQ
        let amqp_url = config.as_ref().amqp_url();
        let conn = lapin::Connection::connect(
            &amqp_url,
            lapin::ConnectionProperties::default(),
        ).await?;
        let channel = conn.create_channel().await?;
        
        // Initialize services
        let queue_service = Arc::new(RabbitMQQueue::new().await?) as Arc<dyn QueueService>;
        
        let task_splitter = Arc::new(RabbitMQTaskSplitter::new(
            channel,
            Arc::clone(&cache_service),
        )) as Arc<dyn TaskSplitterService>;
        
        task_splitter.setup_queues().await?;
        
        // Initialize scrapers
        let zyte_client = ZyteClient::new()?;
        let scraper_service = Self::init_scraper_service(zyte_client, &redis_url).await?;

        Ok(Self {
            config,
            cache_service,
            queue_service,
            scraper_service,
            task_splitter,
        })
    }

    fn build_redis_url() -> Result<String> {
        Ok(format!(
            "redis://{}:{}@{}:{}/",
            std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
            std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
            std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
        ))
    }

    fn mask_password(url: &str) -> String {
        let password = std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set");
        url.replace(&password, "****")
    }

    async fn init_scraper_service(
        zyte_client: ZyteClient,
        redis_url: &str,
    ) -> Result<Arc<dyn ScraperService>> {
        let leroy_scraper = LeroyScraper::new(zyte_client.clone())?;
        let bricodepot_scraper = BricodepotScraper::new(zyte_client.clone());
        let bauhaus_scraper = BauhausScraper::new(zyte_client);

        Ok(Arc::new(CombinedScraperService::new(
            leroy_scraper,
            bauhaus_scraper,
            bricodepot_scraper,
            Box::new(RedisCacheService::new(redis_url).await?),
            Box::new(RabbitMQQueue::new().await?)
        )) as Arc<dyn ScraperService>)
    }
}
