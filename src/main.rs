use rust_scraper::api::start_server;
use redis::aio::ConnectionManager;
use actix_web::web::Data;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let redis_password = std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set");
    let redis_url = format!("redis://:{}@localhost:6379", redis_password);
    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let redis_manager = ConnectionManager::new(redis_client).await.expect("Failed to create Redis connection manager");
    let redis_data = Data::new(redis_manager);

    start_server(redis_data).await
}
