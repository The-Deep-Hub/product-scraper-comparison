use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::models::{store::Store, product::Product};
use derive_more::Display;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Display)]
pub enum TaskStatus {
    #[display(fmt = "pending")]
    Pending,
    #[display(fmt = "processing")]
    Processing,
    #[display(fmt = "completed")]
    Completed,
    #[display(fmt = "failed")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainTask {
    pub id: String,
    pub query: String,
    pub status: TaskStatus,
    pub pending_stores: Vec<Store>,
    pub store_results: HashMap<Store, Vec<Product>>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreTask {
    pub id: String,
    pub main_task_id: String,
    pub query: String,
    pub store: Store,
    pub status: TaskStatus,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoreResult {
    pub task_id: String,
    pub main_task_id: String,
    pub store: Store,
    pub products: Vec<Product>,
    pub created_at: DateTime<Utc>,
}

impl MainTask {
    pub fn new(query: String, store: Option<Store>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            query,
            status: TaskStatus::Pending,
            pending_stores: match store {
                Some(s) => vec![s],
                None => vec![Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot],
            },
            store_results: HashMap::new(),
            error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.pending_stores.is_empty()
    }

    pub fn add_store_result(&mut self, store: Store, products: Vec<Product>) {
        self.store_results.insert(store, products);
        self.pending_stores.retain(|s| *s != store);
        self.updated_at = Utc::now();
        
        if self.is_complete() {
            self.status = TaskStatus::Completed;
        }
    }
}

impl StoreTask {
    pub fn new(main_task_id: String, query: String, store: Store) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            main_task_id,
            query,
            store,
            status: TaskStatus::Pending,
            error: None,
            created_at: now,
            updated_at: now,
        }
    }
} 