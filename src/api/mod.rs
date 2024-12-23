pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod response;
pub mod routes;

pub use config::ApiConfig;
pub use error::ApiError;
pub use response::ApiResponse;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use middleware::auth::AuthMiddleware;
use middleware::logging::setup_logging;
use routes::auth::auth_routes;
use std::net::TcpListener;

pub async fn start_server(listener: TcpListener, auth_middleware: AuthMiddleware) -> std::io::Result<()> {
    setup_logging();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(auth_middleware.clone()))
            .service(auth_routes())
    })
    .listen(listener)?
    .run()
    .await
}
