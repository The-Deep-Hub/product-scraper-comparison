pub mod config;
pub mod models;
pub mod repositories;
pub mod error;

pub use config::MongoConfig;
pub use error::DbError;

use mongodb::Database;
use std::sync::Arc;

#[derive(Clone)]
pub struct MongoDb {
    db: Arc<Database>,
}

impl MongoDb {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
        }
    }

    pub fn get_database(&self) -> &Database {
        &self.db
    }
} 