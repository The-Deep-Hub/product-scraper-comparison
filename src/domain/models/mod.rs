mod error;
mod product;
mod store;

pub use error::DomainError;
pub use product::{Product, ProductId, Price, ProductUrls};
pub use store::Store; 