use std::sync::Arc;
use crate::{
    domain::{
        ports::outbound::{CachePort, QueuePort},
        services::scraper::ScraperService,
    },
};

pub struct AppState {
    pub scraper_service: Arc<ScraperService>,
    pub cache_port: Arc<dyn CachePort>,
    pub queue_port: Arc<dyn QueuePort>,
}

impl AppState {
    pub fn new(
        scraper_service: Arc<ScraperService>,
        cache_port: Arc<dyn CachePort>,
        queue_port: Arc<dyn QueuePort>,
    ) -> Self {
        Self {
            scraper_service,
            cache_port,
            queue_port,
        }
    }
}
