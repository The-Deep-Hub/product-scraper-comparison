use serde::Deserialize;
use tracing::Level;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_level")]
    pub level: String,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub file_output: Option<String>,
}

fn default_level() -> String {
    "info".to_string()
}

fn default_format() -> String {
    "json".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: default_level(),
            format: default_format(),
            file_output: None,
        }
    }
}

impl Config {
    pub fn get_level(&self) -> Level {
        match self.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }
}
