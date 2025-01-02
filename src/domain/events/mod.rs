use crate::domain::models::Product;

#[derive(Debug, Clone)]
pub enum DomainEvent {
    ProductsScraped {
        query: String,
        products: Vec<Product>,
    },
    ProductsCached {
        query: String,
        products: Vec<Product>,
    },
    ScrapeJobEnqueued {
        query: String,
    },
    ScrapeJobCompleted {
        query: String,
        products: Vec<Product>,
    },
} 