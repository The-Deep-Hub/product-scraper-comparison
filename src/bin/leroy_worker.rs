use rust_scraper::{
    clients::zyte::ZyteClient,
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        scraper::ScraperService,
    },
    models::{
        store::Store,
        task::{StoreTask, StoreResult},
    },
    scrapers::LeroyScraper,
};
use tracing::{info, error};
use std::time::Duration;
use tokio::time::sleep;
use futures::StreamExt;
use lapin::{
    options::*, types::FieldTable, BasicProperties,
    Connection, ConnectionProperties, Channel, Consumer,
};
use std::sync::Arc;

const QUEUE_NAME: &str = "store_tasks_leroymerlin";
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const PREFETCH_COUNT: u16 = 3;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,leroy_worker=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize services
    let (cache_service, queue_service, scraper) = initialize_services().await?;
    
    // Get RabbitMQ connection details
    let amqp_url = std::env::var("AMQP_ADDR").expect("AMQP_ADDR must be set");
    
    loop {
        match run_consumer(&amqp_url, cache_service.clone(), queue_service.clone(), scraper.clone()).await {
            Ok(_) => {
                error!("Consumer stopped unexpectedly");
            }
            Err(e) => {
                error!("Consumer error: {}. Reconnecting in {} seconds...", e, RECONNECT_DELAY.as_secs());
            }
        }
        
        sleep(RECONNECT_DELAY).await;
        info!("Attempting to reconnect...");
    }
}

async fn initialize_services() -> Result<(Arc<dyn CacheService>, Arc<dyn QueueService>, Arc<dyn ScraperService>), Box<dyn std::error::Error>> {
    // Initialize Redis cache service
    let redis_url = format!(
        "redis://{}:{}@{}:{}/",
        std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
        std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
        std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
    );
    let cache_service = Arc::new(RedisCacheService::new(&redis_url).await?) as Arc<dyn CacheService>;
    
    // Initialize queue service
    let queue_service = Arc::new(RabbitMQQueue::new().await?) as Arc<dyn QueueService>;
    
    // Initialize Zyte client and Leroy scraper
    let zyte_client = ZyteClient::new()?;
    let scraper = Arc::new(LeroyScraper::new(zyte_client)) as Arc<dyn ScraperService>;
    
    Ok((cache_service, queue_service, scraper))
}

async fn run_consumer(
    amqp_url: &str,
    cache_service: Arc<dyn CacheService>,
    queue_service: Arc<dyn QueueService>,
    scraper: Arc<dyn ScraperService>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to RabbitMQ
    let conn = Connection::connect(
        amqp_url,
        ConnectionProperties::default(),
    ).await?;
    
    let channel = conn.create_channel().await?;
    
    // Set QoS (prefetch)
    channel.basic_qos(PREFETCH_COUNT, BasicQosOptions::default()).await?;
    
    info!("Starting consumer for queue: {}", QUEUE_NAME);
    let mut consumer = channel.basic_consume(
        QUEUE_NAME,
        "leroy_worker",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;
    
    info!("Consumer started successfully. Waiting for messages...");
    
    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let task: StoreTask = serde_json::from_slice(&delivery.data)?;
                info!("Received task: {} for query: {}", task.id, task.query);
                
                match process_task(&task, queue_service.clone(), cache_service.clone(), scraper.clone()).await {
                    Ok(_) => {
                        delivery.ack(BasicAckOptions::default()).await?;
                        info!("Task {} processed successfully", task.id);
                    }
                    Err(e) => {
                        delivery.reject(BasicRejectOptions {
                            requeue: false,
                        }).await?;
                        error!("Failed to process task {}: {}", task.id, e);
                    }
                }
            }
            Err(e) => {
                error!("Error receiving message: {}", e);
            }
        }
    }
    
    Ok(())
}

async fn process_task(
    task: &StoreTask,
    queue_service: Arc<dyn QueueService>,
    cache_service: Arc<dyn CacheService>,
    scraper: Arc<dyn ScraperService>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Scrape products
    let products = scraper.search_products(&task.query).await?;
    info!("Found {} products for query: {}", products.len(), task.query);
    
    // Create store result
    let result = StoreResult {
        task_id: task.id.clone(),
        main_task_id: task.main_task_id.clone(),
        store: Store::LeroyMerlin,
        products,
        created_at: chrono::Utc::now(),
    };
    
    // Update cache with results
    cache_service.set_store_result(&result).await?;
    info!("Stored results for task {}", task.id);
    
    Ok(())
} 