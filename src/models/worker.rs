use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::{
    error::AppError,
    models::store::Store,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub store: Store,
    pub concurrency: u32,
    pub poll_interval_ms: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl FromStr for WorkerConfig {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.as_slice() {
            [store, concurrency] => {
                let store = <Store as FromStr>::from_str(store)
                    .map_err(|e| AppError::ConfigError(format!("Invalid store: {}", e)))?;
                let concurrency = <u32 as FromStr>::from_str(concurrency)
                    .map_err(|_| AppError::ConfigError(format!("Invalid concurrency: {}", concurrency)))?;
                
                Ok(WorkerConfig {
                    store,
                    concurrency,
                    poll_interval_ms: 1000, // Default values
                    max_retries: 3,
                    retry_delay_ms: 5000,
                })
            }
            _ => Err(AppError::ConfigError("Worker config must be in format 'store:concurrency'".to_string())),
        }
    }
}

impl WorkerConfig {
    pub fn with_poll_interval(mut self, ms: u64) -> Self {
        self.poll_interval_ms = ms;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_retry_delay(mut self, ms: u64) -> Self {
        self.retry_delay_ms = ms;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStats {
    pub store: Store,
    pub last_task_time: Option<chrono::DateTime<chrono::Utc>>,
    pub status: WorkerStatus,
    pub tasks_processed: u64,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerStatus {
    Running,
    Idle,
    Error(String),
    Stopped,
}
