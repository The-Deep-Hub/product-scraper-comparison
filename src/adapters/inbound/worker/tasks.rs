use crate::domain::models::store::Store;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreTask {
    pub id: String,
    pub main_task_id: String,
    pub query: String,
    pub store: Store,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreResult {
    pub task_id: String,
    pub main_task_id: String,
    pub store: Store,
    pub products: Vec<crate::domain::models::product::Product>,
    pub created_at: DateTime<Utc>,
} 