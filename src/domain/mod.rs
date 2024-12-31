pub mod models;
pub mod ports;
pub mod services;
pub mod events;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use models::{Product, Store, DomainError}; 