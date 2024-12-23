use mongodb::Client;
use redis::aio::ConnectionManager;
use std::env;
use rust_scraper::api::start_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let db_name = env::var("MONGO_DATABASE").expect("MONGO_DATABASE must be set");

    // Construct MongoDB URI
    let mongo_uri = format!(
        "mongodb://{}:{}@{}:{}/{}",
        env::var("MONGO_APP_USERNAME").expect("MONGO_APP_USERNAME must be set"),
        env::var("MONGO_APP_PASSWORD").expect("MONGO_APP_PASSWORD must be set"),
        env::var("MONGO_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("MONGO_PORT").unwrap_or_else(|_| "27017".to_string()),
        db_name
    );

    // Construct Redis URI
    let redis_uri = format!(
        "redis://:{}@{}:{}",
        env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
        env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string())
    );

    // Construct RabbitMQ URI
    let rabbitmq_uri = format!(
        "amqp://{}:{}@{}:{}{}",
        env::var("RABBITMQ_USER").expect("RABBITMQ_USER must be set"),
        env::var("RABBITMQ_PASSWORD").expect("RABBITMQ_PASSWORD must be set"),
        env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string()),
        env::var("RABBITMQ_VHOST").unwrap_or_else(|_| "/".to_string())
    );

    let mongo_client = Client::with_uri_str(&mongo_uri).await.expect("Failed to connect to MongoDB");
    let mongo_db = mongo_client.database(&db_name);
    let redis_client = redis::Client::open(redis_uri).expect("Failed to connect to Redis");
    let redis_manager = ConnectionManager::new(redis_client).await.expect("Failed to create Redis connection manager");

    start_server(mongo_db, redis_manager, &rabbitmq_uri).await
}
