use serde::{Serialize, Deserialize};
use crate::error::{AppError, AppResult};
use clap::ValueEnum;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
pub enum Store {
    #[serde(rename = "bauhaus")]
    #[clap(name = "bauhaus")]
    Bauhaus,
    
    #[serde(rename = "bricodepot")]
    #[clap(name = "bricodepot")]
    Bricodepot,
    
    #[serde(rename = "leroy")]
    #[clap(name = "leroy")]
    LeroyMerlin,
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
            Err(AppError::BadRequest(format!("Invalid store URL: {}", url)))
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            Store::Bauhaus => "https://www.bauhaus.es",
            Store::Bricodepot => "https://www.bricodepot.es",
            Store::LeroyMerlin => "https://www.leroymerlin.es",
        }
    }
}

impl FromStr for Store {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bauhaus" => Ok(Store::Bauhaus),
            "bricodepot" => Ok(Store::Bricodepot),
            "leroy" => Ok(Store::LeroyMerlin),
            _ => Err(format!("Unknown store: {}", s)),
        }
    }
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Bauhaus => write!(f, "bauhaus"),
            Store::Bricodepot => write!(f, "bricodepot"),
            Store::LeroyMerlin => write!(f, "leroy"),
        }
    }
}