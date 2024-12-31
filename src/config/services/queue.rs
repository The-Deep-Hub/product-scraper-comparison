use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub amqp_addr: String,
    #[serde(default = "default_prefetch_count")]
    pub prefetch_count: u16,
}

fn default_prefetch_count() -> u16 {
    1
}
