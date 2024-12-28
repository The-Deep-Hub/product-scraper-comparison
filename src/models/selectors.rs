use std::collections::HashMap;
use scraper::Selector;
use crate::error::{AppError, AppResult};

/// Represents a collection of CSS selectors used for web scraping
#[derive(Debug)]
pub struct Selectors {
    selectors: HashMap<String, Selector>,
}

impl Selectors {
    /// Creates a new Selectors instance from a HashMap of selector strings
    pub fn new(selector_map: HashMap<&str, &str>) -> AppResult<Self> {
        let mut selectors = HashMap::new();
        
        for (key, value) in selector_map {
            let selector = Selector::parse(value)
                .map_err(|e| AppError::BadRequest(format!("Failed to parse selector '{}': {}", value, e)))?;
            selectors.insert(key.to_string(), selector);
        }
        
        Ok(Self { selectors })
    }

    /// Gets a reference to a selector by its key
    pub fn get(&self, key: &str) -> Option<&Selector> {
        self.selectors.get(key)
    }

    /// Returns true if the selectors collection contains a selector with the given key
    pub fn contains(&self, key: &str) -> bool {
        self.selectors.contains_key(key)
    }
}
