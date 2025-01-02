pub mod inbound;
pub mod outbound;

// Re-export commonly used adapters
pub use outbound::scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper}; 