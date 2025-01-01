use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Store {
    #[serde(rename = "leroy")]
    LeroyMerlin,
    #[serde(rename = "bauhaus")]
    Bauhaus,
    #[serde(rename = "bricodepot")]
    Bricodepot,
}

impl Default for Store {
    fn default() -> Self {
        Store::LeroyMerlin
    }
}

impl Store {
    pub fn as_str(&self) -> &'static str {
        match self {
            Store::LeroyMerlin => "leroy",
            Store::Bauhaus => "bauhaus",
            Store::Bricodepot => "bricodepot",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "leroy" => Some(Store::LeroyMerlin),
            "bauhaus" => Some(Store::Bauhaus),
            "bricodepot" => Some(Store::Bricodepot),
            _ => None,
        }
    }
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
} 