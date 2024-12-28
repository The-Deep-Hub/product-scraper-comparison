use std::sync::Arc;
use clap::Parser;
use tokio::time::Duration;
use tracing::{info, error};

use rust_scraper::{
    models::{
        store::Store,
        worker::{WorkerConfig, WorkerStats, WorkerStatus},
        task::{StoreTask, StoreResult},
    },
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        scraper::ScraperService,
    },
    scrapers::{BauhausScraper, BricodepotScraper, LeroyScraper},
    clients::zyte::ZyteClient,
    error::{AppError, AppResult},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Stores to process with their concurrency (format: store:concurrency, e.g., bauhaus:3,bricodepot:2)
    #[arg(short, long, value_delimiter = ',')]
    stores: Vec<WorkerConfig>,

    #[arg(short, long, default_value = "1000")]
    poll_interval: u64,

    #[arg(short, long, default_value = "3")]
    max_retries: u32,

    #[arg(short, long, default_value = "5000")]
    retry_delay: u64,
}

struct StoreWorker {
    store: Store,
    cache_service: Arc<dyn CacheService>,
    queue_service: Arc<dyn QueueService>,
    scraper: Arc<dyn ScraperService + Send + Sync>,
    stats: WorkerStats,
}

async fn create_store_worker(
    config: &WorkerConfig,
    cache_service: Arc<dyn CacheService>,
    queue_service: Arc<dyn QueueService>,
    zyte_client: Arc<ZyteClient>,
) -> AppResult<StoreWorker> {
    let scraper: Arc<dyn ScraperService + Send + Sync> = match config.store {
        Store::Bauhaus => Arc::new(BauhausScraper::new((*zyte_client).clone())),
        Store::Bricodepot => Arc::new(BricodepotScraper::new((*zyte_client).clone())),
        Store::LeroyMerlin => Arc::new(LeroyScraper::new((*zyte_client).clone())?),
    };

    Ok(StoreWorker {
        store: config.store,
        cache_service,
        queue_service,
        scraper,
        stats: WorkerStats {
            store: config.store,
            last_task_time: None,
            status: WorkerStatus::Idle,
            tasks_processed: 0,
            errors: 0,
        },
    })
}

async fn process_tasks(mut worker: StoreWorker, config: WorkerConfig) -> AppResult<()> {
    let queue_name = format!("store_tasks_{}", worker.store.to_string().to_lowercase());
    info!(
        "[{:?}-{:02}] Starting worker thread for queue {}",
        worker.store, worker.stats.store, queue_name
    );

    loop {
        match worker.queue_service.consume(&queue_name).await {
            Ok(Some(message)) => {
                let message = message.downcast::<Vec<u8>>()
                    .map_err(|_| AppError::QueueError("Failed to downcast message".into()))?;
                
                let task: StoreTask = serde_json::from_slice(&message)
                    .map_err(|e| AppError::QueueError(format!("Failed to deserialize task: {}", e)))?;

                info!(
                    "[{:?}-{:02}] Processing task {} for store {:?}",
                    worker.store, worker.stats.store, task.id, worker.store
                );

                worker.stats.status = WorkerStatus::Running;

                match worker.scraper.search_products(&task.query).await {
                    Ok(products) => {
                        info!("Found {} products for query: {}", products.len(), task.query);
                        let result = StoreResult {
                            task_id: task.id.clone(),
                            main_task_id: task.main_task_id.clone(),
                            store: worker.store,
                            products,
                            created_at: chrono::Utc::now(),
                        };
                        if let Err(e) = worker.cache_service.set_store_result(&result).await {
                            error!("Failed to cache results for task {}: {}", task.id, e);
                        }
                        worker.queue_service.mark_task_completed(task.id).await?;
                        worker.stats.tasks_processed += 1;
                        worker.stats.status = WorkerStatus::Idle;
                    }
                    Err(e) => {
                        error!("Failed to process task {}: {}", task.id, e);
                        worker.queue_service.mark_task_failed(task.id, e.to_string()).await?;
                        worker.stats.errors += 1;
                        worker.stats.status = WorkerStatus::Error(e.to_string());
                    }
                }
            }
            Ok(None) => {
                info!(
                    "[{:?}-{:02}] No pending tasks, waiting...",
                    worker.store, worker.stats.store
                );
                tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;
            }
            Err(e) => {
                error!(
                    "[{:?}-{:02}] Failed to get pending task: {}",
                    worker.store, worker.stats.store, e
                );
                worker.stats.status = WorkerStatus::Error(e.to_string());
                tokio::time::sleep(Duration::from_millis(config.retry_delay_ms)).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging with a custom format
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug")
        .with_thread_ids(true)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Parse command line arguments
    let args = Args::parse();
    
    info!("Starting workers with the following configuration:");
    for config in &args.stores {
        info!(
            "- Store {:?}: {} workers (poll: {}ms, retries: {}, retry delay: {}ms)",
            config.store,
            config.concurrency,
            config.poll_interval_ms,
            config.max_retries,
            config.retry_delay_ms
        );
    }

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

    // Initialize shared services
    let cache_service: Arc<dyn CacheService> = Arc::new(RedisCacheService::new(&redis_url).await?);
    let queue_service: Arc<dyn QueueService> = Arc::new(RabbitMQQueue::new().await?);
    let zyte_client = Arc::new(ZyteClient::new()?);

    // Create a worker handle for each store and its specified concurrency
    let mut handles = Vec::new();
    
    for mut config in args.stores {
        // Update config with CLI args
        config = config
            .with_poll_interval(args.poll_interval)
            .with_max_retries(args.max_retries)
            .with_retry_delay(args.retry_delay);

        info!("Initializing {:?} store workers...", config.store);
        
        for _ in 0..config.concurrency {
            let worker = create_store_worker(
                &config,
                cache_service.clone(),
                queue_service.clone(),
                zyte_client.clone(),
            ).await?;
            
            let config = config.clone();
            let store = config.store; // Clone store before moving config
            let handle = tokio::spawn(async move {
                if let Err(e) = process_tasks(worker, config).await {
                    error!(
                        "[{:?}-{:02}] Worker thread failed: {}",
                        store, 0, e
                    );
                }
            });
            
            handles.push(handle);
        }
    }

    info!("All workers initialized and running. Press Ctrl+C to stop.");

    // Wait for all worker threads
    for handle in handles {
        handle.await.map_err(|e| AppError::WorkerError(e.to_string()))?;
    }

    Ok(())
}
