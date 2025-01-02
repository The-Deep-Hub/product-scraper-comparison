pub mod outbound;
pub mod inbound;

// Re-export commonly used ports
pub use outbound::{CachePort, ScraperPort, QueuePort, HttpClientPort};
pub use inbound::ProductSearchPort; 