use std::time::Duration;

/// Queue configuration port
pub trait QueueConfig: Send + Sync {
    /// Get the queue connection URL
    fn connection_url(&self) -> String;
    
    /// Get the queue name
    fn queue_name(&self) -> String;
    
    /// Get the connection timeout
    fn connection_timeout(&self) -> Duration;
    
    /// Get the consumer prefetch count
    fn prefetch_count(&self) -> u16;
    
    /// Get retry settings
    fn retry_interval(&self) -> Duration;
    fn max_retries(&self) -> u32;
} 