use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_prefetch_count")]
    pub prefetch_count: u16,
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay_secs: u64,
    #[serde(default = "default_product_limit")]
    pub product_limit: usize,
    #[serde(default = "default_store_queues")]
    pub store_queues: HashMap<String, String>,
}

fn default_prefetch_count() -> u16 {
    3
}

fn default_reconnect_delay() -> u64 {
    5
}

fn default_product_limit() -> usize {
    100
}

fn default_store_queues() -> HashMap<String, String> {
    let mut queues = HashMap::new();
    queues.insert("leroy".to_string(), "leroy_tasks".to_string());
    queues.insert("bauhaus".to_string(), "bauhaus_tasks".to_string());
    queues.insert("bricodepot".to_string(), "bricodepot_tasks".to_string());
    queues
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefetch_count: default_prefetch_count(),
            reconnect_delay_secs: default_reconnect_delay(),
            product_limit: default_product_limit(),
            store_queues: default_store_queues(),
        }
    }
}

impl Config {
    pub fn get_store_queues(&self) -> Vec<String> {
        self.store_queues.values().cloned().collect()
    }

    pub fn get_product_limit(&self) -> Option<usize> {
        Some(self.product_limit)
    }
} 