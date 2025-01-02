use std::time::Duration;

/// Database configuration port
pub trait DatabaseConfig: Send + Sync {
    /// Get the database connection string
    fn connection_string(&self) -> String;
    
    /// Get the database name
    fn database_name(&self) -> String;
    
    /// Get the connection timeout
    fn connection_timeout(&self) -> Duration;
    
    /// Get the maximum pool size
    fn max_pool_size(&self) -> u32;
} 