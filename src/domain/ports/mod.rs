pub mod inbound;
pub mod outbound;

// Re-export commonly used ports
pub use inbound::ProductSearchPort;
pub use outbound::{CachePort, ScraperPort, QueuePort, HttpClientPort}; 