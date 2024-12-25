use rust_scraper::services::queue::{QueueService, RabbitMQQueue};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,publish_task=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize queue service
    let queue_service = RabbitMQQueue::new().await?;
    
    // Create a task
    let query = "taladro"; // You can change this query
    let task_id = queue_service.create_task(query.to_string()).await?;
    
    info!("Created task with ID: {} for query: {}", task_id, query);

    Ok(())
} 