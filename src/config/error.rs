use std::error::Error;
use std::fmt;
use config::ConfigError as ConfError;

#[derive(Debug)]
pub enum ConfigError {
    Config(ConfError),
    Missing(String),
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Config(err) => write!(f, "Configuration error: {}", err),
            ConfigError::Missing(field) => write!(f, "Missing required field: {}", field),
            ConfigError::Invalid(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Config(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ConfError> for ConfigError {
    fn from(err: ConfError) -> Self {
        ConfigError::Config(err)
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;
