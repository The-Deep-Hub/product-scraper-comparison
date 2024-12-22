use config::{Config, ConfigError, Environment as ConfigEnvironment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub environment: ApiEnvironment,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiEnvironment {
    Development,
    Production,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_allowed_origins: vec!["http://localhost:3000".to_string()],
            environment: ApiEnvironment::Development,
        }
    }
}

impl ApiConfig {
    pub fn new() -> Self {
        Self::load().unwrap_or_else(|err| {
            eprintln!("Failed to load configuration: {}. Using defaults.", err);
            Self::default()
        })
    }

    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start with default settings
            .set_default("environment", "development")?
            // Add config file if it exists
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add environment variables with prefix "APP_"
            .add_source(ConfigEnvironment::with_prefix("APP"))
            .build()?;

        s.try_deserialize()
    }

    pub fn is_development(&self) -> bool {
        self.environment == ApiEnvironment::Development
    }
} 