pub mod middleware;
pub mod models;
pub mod routes;
pub mod utils;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use middleware::AuthMiddleware;
use mongodb::Client;
use redis::aio::ConnectionManager;

pub async fn start_server(_redis_manager: web::Data<ConnectionManager>) -> std::io::Result<()> {
    // Connect to MongoDB
    let mongo_uri = std::env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string());
    let mongo_client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to create MongoDB client");
    let db = mongo_client.database("rust_scraper");
    let user_repo = web::Data::new(crate::db::repositories::user::UserRepository::new(db));

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(user_repo.clone())
            .service(
                web::scope("/api")
                    .configure(routes::auth_config)
                    .service(
                        web::scope("/protected")
                            .wrap(AuthMiddleware)
                            .configure(routes::health_config)
                    )
            )
    })
    .bind("127.0.0.1:8080")?;

    println!("Server running at http://127.0.0.1:8080");
    server.run().await
}
