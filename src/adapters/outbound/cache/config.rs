use serde::Deserialize;

/// Redis adapter configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_user")]
    pub user: String,
    #[serde(default)]
    pub password: String,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    6379
}

fn default_user() -> String {
    "default".to_string()
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            user: default_user(),
            password: String::new(),
        }
    }
}

impl RedisConfig {
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.password.is_empty() {
            return Err("Redis password is not configured".to_string());
        }
        Ok(())
    }

    /// Builds the Redis URL
    pub fn get_url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}/",
            self.user,
            self.password,
            self.host,
            self.port,
        )
    }
} 