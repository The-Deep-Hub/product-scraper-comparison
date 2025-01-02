pub mod config;
pub mod error;
pub mod domain;
pub mod adapters;

pub use error::{AppError, AppResult};
pub use adapters::inbound::{TaskProcessor, StoreTask, StoreResult};
