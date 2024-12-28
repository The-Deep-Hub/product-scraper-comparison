use std::sync::Arc;
use clap::Parser;
use tokio::time::Duration;
use tracing::{info, error};

use rust_scraper::{
    models::store::Store,
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        scraper::{ScraperService, CombinedScraperService},
    },
    scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper},
    clients::zyte::ZyteClient,
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Store to process (bauhaus, bricodepot, leroy)
    #[clap(long)]
    store: Store,

    /// Number of concurrent tasks to process
    #[clap(long, default_value = "1")]
    concurrency: u32,

    /// Time to wait between processing tasks (in milliseconds)
    #[clap(long, default_value = "1000")]
    poll_interval: u64,
}

async fn initialize_services(store: Store) -> Result<(Arc<dyn CacheService>, Arc<dyn QueueService>, Arc<dyn ScraperService>), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize Redis connection
    let redis_url = format!(
        "redis://{}:{}@{}:{}/",
        std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
        std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
        std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
    );
    info!("Connecting to Redis...");
    let cache_service = Arc::new(RedisCacheService::new(&redis_url).await?) as Arc<dyn CacheService>;

    // Initialize queue service
    info!("Initializing queue service...");
    let queue_service = Arc::new(RabbitMQQueue::new().await?) as Arc<dyn QueueService>;

    // Initialize Zyte client
    info!("Initializing Zyte client...");
    let zyte_client = Arc::new(ZyteClient::new()?);

    // Initialize store-specific scraper
    info!("Initializing {} scraper...", store);
    let scraper: Arc<dyn ScraperService> = match store {
        Store::Bauhaus => Arc::new(BauhausScraper::new(zyte_client.clone())),
        Store::Bricodepot => Arc::new(BricodepotScraper::new(zyte_client.clone())),
        Store::LeroyMerlin => Arc::new(LeroyScraper::new(zyte_client.clone())),
    };

    Ok((cache_service, queue_service, scraper))
}

async fn process_tasks(
    store: Store,
    cache_service: Arc<dyn CacheService>,
    queue_service: Arc<dyn QueueService>,
    scraper: Arc<dyn ScraperService>,
    poll_interval: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting {} worker...", store);

    loop {
        match queue_service.get_pending_task().await {
            Ok(Some(task)) => {
                info!("Processing task {} for store {}", task.id, store);
                
                // Mark task as processing
                if let Err(e) = queue_service.mark_task_processing(task.id.clone()).await {
                    error!("Failed to mark task as processing: {}", e);
                    continue;
                }

                // Process the task
                match scraper.search_store_products(store, &task.query).await {
                    Ok(products) => {
                        info!("Found {} products for task {}", products.len(), task.id);
                        
                        // Cache the results
                        if let Err(e) = cache_service.set_store_products(&task.id, store, &products).await {
                            error!("Failed to cache results: {}", e);
                            queue_service.mark_task_failed(task.id, format!("Failed to cache results: {}", e)).await?;
                            continue;
                        }

                        // Mark task as completed
                        queue_service.mark_task_completed(task.id).await?;
                    }
                    Err(e) => {
                        error!("Failed to process task {}: {}", task.id, e);
                        queue_service.mark_task_failed(task.id, e.to_string()).await?;
                    }
                }
            }
            Ok(None) => {
                tokio::time::sleep(poll_interval).await;
            }
            Err(e) => {
                error!("Failed to get pending task: {}", e);
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();
    info!("Starting worker for store: {:?}", args.store);

    // Initialize services
    let (cache_service, queue_service, scraper) = initialize_services(args.store).await?;

    // Start processing tasks
    let poll_interval = Duration::from_millis(args.poll_interval);
    
    let mut handles = Vec::new();
    for i in 0..args.concurrency {
        info!("Starting worker thread {}", i);
        let cache_service = cache_service.clone();
        let queue_service = queue_service.clone();
        let scraper = scraper.clone();
        let store = args.store;
        
        let handle = tokio::spawn(async move {
            if let Err(e) = process_tasks(
                store,
                cache_service,
                queue_service,
                scraper,
                poll_interval,
            ).await {
                error!("Worker thread {} failed: {}", i, e);
            }
        });
        
        handles.push(handle);
    }

    // Wait for all worker threads
    for handle in handles {
        handle.await?;
    }

    Ok(())
}
