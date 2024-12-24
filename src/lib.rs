pub mod api;
pub mod clients;
pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod scrapers;
pub mod services;
pub mod utils;

pub use error::{AppError, AppResult};
