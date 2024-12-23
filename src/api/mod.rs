pub mod middleware;
pub mod models;
pub mod routes;
pub mod utils;

use actix_web::{web, App, HttpServer};
use mongodb::Database;
use redis::aio::ConnectionManager;

use crate::{
    db::repositories::{
        user::UserRepository,
        scraping::ScrapingRepository,
    },
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQService},
    },
};

pub async fn start_server(
    db: Database,
    redis: ConnectionManager,
    rabbitmq_uri: &str,
) -> std::io::Result<()> {
    let user_repository = UserRepository::new(db.clone());
    let scraping_repository = ScrapingRepository::new(db.clone());
    let cache_service = web::Data::new(Box::new(RedisCacheService::new(redis)) as Box<dyn CacheService>);
    let queue_service = web::Data::new(Box::new(RabbitMQService::new(rabbitmq_uri).await.unwrap()) as Box<dyn QueueService>);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(user_repository.clone()))
            .app_data(web::Data::new(scraping_repository.clone()))
            .app_data(cache_service.clone())
            .app_data(queue_service.clone())
            .service(
                web::scope("/api")
                    .wrap(middleware::auth::AuthMiddleware)
                    .configure(routes::auth::config)
                    .configure(routes::health::config)
                    .configure(routes::scraper::config)
                    .configure(routes::job::config)
                    .configure(routes::results::config)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
