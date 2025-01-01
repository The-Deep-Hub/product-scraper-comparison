use rust_scraper::config::builder::AppConfig;
use lapin::{
    options::*, types::FieldTable,
    Connection, ConnectionProperties,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,setup=debug")
        .init();

    // Load configuration
    let config = AppConfig::new()?;

    // Connect to RabbitMQ
    info!("Connecting to RabbitMQ at {}", config.amqp_url());
    let conn = Connection::connect(
        &config.amqp_url(),
        ConnectionProperties::default(),
    ).await?;

    let channel = conn.create_channel().await?;

    // Get queue names from worker config
    let queue_names = config.worker.get_store_queues();

    // Create queues
    for queue_name in queue_names {
        info!("Creating queue: {}", queue_name);
        channel.queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                ..Default::default()
            },
            FieldTable::default()
        ).await?;
    }

    info!("Setup completed successfully!");
    Ok(())
} 