mod cache;
mod scraper;
mod queue;
mod http_client;

pub use cache::CachePort;
pub use scraper::ScraperPort;
pub use queue::{QueuePort, ScrapeJob, JobPriority};
pub use http_client::{HttpClientPort, HttpResponse}; 