use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl Config {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
