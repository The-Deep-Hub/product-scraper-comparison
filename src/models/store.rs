use serde::{Serialize, Deserialize};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Store {
    #[serde(rename = "leroy")]
    LeroyMerlin,
    #[serde(rename = "bauhaus")]
    Bauhaus,
    #[serde(rename = "bricodepot")]
    Bricodepot,
}

impl Store {
    pub fn from_url(url: &str) -> AppResult<Self> {
        let url = url.to_lowercase();
        if url.contains("leroymerlin") {
            Ok(Store::LeroyMerlin)
        } else if url.contains("bauhaus") {
            Ok(Store::Bauhaus)
        } else if url.contains("bricodepot") {
            Ok(Store::Bricodepot)
        } else {
            Err(AppError::BadRequest("Unknown store URL".into()))
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            Store::LeroyMerlin => "https://www.leroymerlin.es",
            Store::Bauhaus => "https://www.bauhaus.es",
            Store::Bricodepot => "https://www.bricodepot.es",
        }
    }
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::LeroyMerlin => write!(f, "leroymerlin"),
            Store::Bauhaus => write!(f, "bauhaus"),
            Store::Bricodepot => write!(f, "bricodepot"),
        }
    }
} 