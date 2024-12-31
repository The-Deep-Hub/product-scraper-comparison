use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub mongodb_uri: String,
    #[serde(default = "default_redis_host")]
    pub redis_host: String,
    #[serde(default = "default_redis_port")]
    pub redis_port: u16,
    #[serde(default = "default_redis_user")]
    pub redis_user: String,
    #[serde(default)]
    pub redis_password: String,
}

fn default_redis_host() -> String {
    "localhost".to_string()
}

fn default_redis_port() -> u16 {
    6379
}

fn default_redis_user() -> String {
    "default".to_string()
}
