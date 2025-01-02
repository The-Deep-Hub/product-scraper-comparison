use std::time::Duration;

/// Cache configuration port
pub trait CacheConfig: Send + Sync {
    /// Get the cache connection URL
    fn connection_url(&self) -> String;
    
    /// Get the default TTL for cached items
    fn default_ttl(&self) -> Duration;
    
    /// Get the connection timeout
    fn connection_timeout(&self) -> Duration;
    
    /// Get the maximum number of connections
    fn max_connections(&self) -> u32;
} 