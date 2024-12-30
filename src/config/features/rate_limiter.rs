use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_max_requests")]
    pub max_requests: u32,
    #[serde(default = "default_window_size")]
    pub window_size_seconds: u64,
    #[serde(default = "default_retry_after")]
    pub retry_after_seconds: u64,
}

fn default_max_requests() -> u32 { 100 }
fn default_window_size() -> u64 { 60 }  // 1 minute
fn default_retry_after() -> u64 { 5 }   // 5 seconds

impl Default for Config {
    fn default() -> Self {
        Self {
            max_requests: default_max_requests(),
            window_size_seconds: default_window_size(),
            retry_after_seconds: default_retry_after(),
        }
    }
}

impl Config {
    pub fn window_size(&self) -> Duration {
        Duration::from_secs(self.window_size_seconds)
    }

    pub fn retry_after(&self) -> Duration {
        Duration::from_secs(self.retry_after_seconds)
    }
}
