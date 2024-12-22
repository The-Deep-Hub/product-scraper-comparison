pub mod health;

use actix_web::web;

pub trait ApiRouteConfig {
    fn configure() -> Box<dyn Fn(&mut web::ServiceConfig)>;
}

// Re-export health config for convenience
pub use health::health_config; 