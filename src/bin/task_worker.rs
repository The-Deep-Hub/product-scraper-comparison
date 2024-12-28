   use rust_scraper::{
    clients::zyte::ZyteClient,
    error::AppError,
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        scraper::ScraperService,
    },
    scrapers::{LeroyScraper, BauhausScraper, BricodepotScraper},
    models::store::Store,
};
use tracing::{info, error, debug};
use std::time::Duration;
use tokio::time::sleep;
use futures::StreamExt;
use lapin::{
    options::*, types::FieldTable,
    Connection, ConnectionProperties,
};
use std::sync::Arc;
use uuid::Uuid;
use std::env;

const PREFETCH_COUNT: u16 = 3;
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const QUEUE_NAME: &str = "scraper_tasks";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,task_worker=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize services
    let (cache_service, queue_service, scrapers) = initialize_services().await?;
    
    // Get RabbitMQ connection details
    let amqp_url = std::env::var("AMQP_ADDR").expect("AMQP_ADDR must be set");
    
    loop {
        match setup_and_run_consumer(&amqp_url, cache_service.clone(), queue_service.clone(), &scrapers).await {
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

async fn initialize_services() -> Result<(Arc<dyn CacheService>, Arc<dyn QueueService>, Vec<Arc<dyn ScraperService>>), Box<dyn std::error::Error>> {
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
    
    // Initialize Zyte client and scrapers
    let zyte_client = ZyteClient::new()?;
    let scrapers: Vec<Arc<dyn ScraperService>> = vec![
        Arc::new(BauhausScraper::new(zyte_client.clone())),
        Arc::new(BricodepotScraper::new(zyte_client.clone())),
        Arc::new(LeroyScraper::new(zyte_client.clone())?),
    ];
    
    Ok((cache_service, queue_service, scrapers))
}

async fn setup_and_run_consumer(
    amqp_url: &str,
    cache_service: Arc<dyn CacheService>,
    queue_service: Arc<dyn QueueService>,
    scrapers: &[Arc<dyn ScraperService>],
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
        "task_worker",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;
    
    info!("Consumer started successfully. Waiting for messages...");
    
    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let task_id = String::from_utf8_lossy(&delivery.data);
                info!("Received task: {}", task_id);
                
                match process_task(queue_service.clone(), cache_service.clone(), scrapers).await {
                    Ok(_) => {
                        delivery.ack(BasicAckOptions::default()).await?;
                        info!("Task {} processed successfully", task_id);
                    }
                    Err(e) => {
                        delivery.reject(BasicRejectOptions {
                            requeue: false,
                        }).await?;
                        error!("Failed to process task {}: {}", task_id, e);
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
    queue_service: Arc<dyn QueueService>,
    cache_service: Arc<dyn CacheService>,
    scrapers: &[Arc<dyn ScraperService>],
) -> Result<(), Box<dyn std::error::Error>> {
    let task = queue_service.get_pending_task().await?
        .ok_or_else(|| AppError::TaskError("No pending task found".to_string()))?;
    info!("Processing task {} for query: {}", task.id, task.query);
    
    debug!("Splitting task {} into store-specific tasks", task.id);
    info!("Splitting task {} for stores: [LeroyMerlin, Bauhaus, Bricodepot]", task.id);
    
    let mut all_products = Vec::new();
    let mut had_error = false;
    
    // Create store-specific tasks
    for store in &[Store::Bauhaus, Store::Bricodepot, Store::LeroyMerlin] {
        let task_id = Uuid::new_v4().to_string();
        let queue_name = format!("store_tasks_{}", store.to_string().to_lowercase());
        info!("Published task {} to queue {}", task_id, queue_name);

        match queue_service.publish(&queue_name, Box::new(task.query.clone())).await {
            Ok(_) => {
                match scrapers.iter().find(|s| s.get_store() == *store) {
                    Some(scraper) => {
                        match scraper.search_products(&task.query).await {
                            Ok(products) => {
                                info!("Found {} products from {}", products.len(), store);
                                all_products.extend(products);
                            }
                            Err(e) => {
                                error!("Error scraping {}: {}", store, e);
                                had_error = true;
                            }
                        }
                    }
                    None => {
                        error!("No scraper found for store {:?}", store);
                        had_error = true;
                    }
                }
            }
            Err(e) => {
                error!("Error publishing task to queue {}: {}", queue_name, e);
                had_error = true;
            }
        }
    }

    info!("Split task {} into store-specific tasks", task.id);
    
    // Cache results if we have any
    if !all_products.is_empty() {
        if let Err(e) = cache_service.set_search_results(&task.query, &all_products).await {
            error!("Error caching results: {}", e);
            had_error = true;
        }
    }
    
    // Update task status
    if had_error {
        queue_service.mark_task_failed(task.id.to_string(), "Some scrapers failed".into()).await?;
    } else {
        queue_service.mark_task_completed(task.id.to_string()).await?;
        info!("Task completed, returning {} products", all_products.len());
    }
    
    Ok(())
}