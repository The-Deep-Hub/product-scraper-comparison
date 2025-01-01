use std::sync::Arc;
use crate::domain::{
    ports::outbound::{CachePort, QueuePort},
    services::scraper::ScraperService,
};

pub struct AppState<C, Q>
where
    C: CachePort + 'static,
    Q: QueuePort + 'static,
{
    pub scraper_service: Arc<ScraperService>,
    pub cache_port: Arc<C>,
    pub queue_port: Arc<Q>,
}

impl<C, Q> AppState<C, Q>
where
    C: CachePort + 'static,
    Q: QueuePort + 'static,
{
    pub fn new(
        scraper_service: Arc<ScraperService>,
        cache_port: Arc<C>,
        queue_port: Arc<Q>,
    ) -> Self {
        Self {
            scraper_service,
            cache_port,
            queue_port,
        }
    }
} 