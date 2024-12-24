use rust_scraper::{
    clients::zyte::ZyteClient,
    services::{
        cache::{CacheService, RedisCacheService},
        queue::{QueueService, RabbitMQQueue},
        scraper::{CombinedScraperService, ScraperService},
    },
    models::store::Store,
    scrapers::{LeroyScraper, BauhausScraper, BricodepotScraper},
};
use tracing::{info, error};
use std::time::Duration;
use tokio::time::sleep;
use futures::future::join_all;
use std::sync::Arc;

const MAX_CONCURRENT_TASKS: usize = 3;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,task_worker=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

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
    
    // Initialize Zyte client
    let zyte_client = ZyteClient::new()?;

    // Initialize scrapers
    let leroy_scraper = Arc::new(LeroyScraper::new(zyte_client.clone()));
    let bauhaus_scraper = Arc::new(BauhausScraper::new(zyte_client.clone()));
    let bricodepot_scraper = Arc::new(BricodepotScraper::new(zyte_client));

    info!("Task worker started. Waiting for tasks...");

    let mut running_tasks = Vec::new();

    loop {
        // Clean up completed tasks
        running_tasks.retain(|task: &tokio::task::JoinHandle<()>| !task.is_finished());

        // If we have capacity, get more tasks
        while running_tasks.len() < MAX_CONCURRENT_TASKS {
            if let Ok(Some(task)) = queue_service.get_pending_task().await {
                info!("Starting task {} - Query: {}", task.id, task.query);
                
                // Clone the services for the task
                let queue_service = queue_service.clone();
                let cache_service = cache_service.clone();
                let leroy_scraper = leroy_scraper.clone();
                let bauhaus_scraper = bauhaus_scraper.clone();
                let bricodepot_scraper = bricodepot_scraper.clone();
                
                // Spawn a new task
                let handle = tokio::spawn(async move {
                    // Create futures for each store
                    let store_futures: Vec<_> = vec![
                        Box::pin(scrape_store_dyn(Store::LeroyMerlin, leroy_scraper.as_ref(), &task.query)),
                        Box::pin(scrape_store_dyn(Store::Bauhaus, bauhaus_scraper.as_ref(), &task.query)),
                        Box::pin(scrape_store_dyn(Store::Bricodepot, bricodepot_scraper.as_ref(), &task.query)),
                    ];
                    
                    // Run all scraping tasks in parallel
                    let results = join_all(store_futures).await;
                    
                    // Combine results
                    let mut all_products = Vec::new();
                    let mut had_error = false;
                    
                    for (store, result) in [Store::LeroyMerlin, Store::Bauhaus, Store::Bricodepot].iter().zip(results) {
                        match result {
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

                    // Cache combined results if we have any products
                    if !all_products.is_empty() {
                        if let Err(e) = cache_service.set_search_results(&task.query, &all_products).await {
                            error!("Error caching results for {}: {}", task.query, e);
                            had_error = true;
                        }
                    }
                    
                    // Mark task status
                    if had_error {
                        if let Err(e) = queue_service.mark_task_failed(task.id.clone(), "Some stores failed to scrape".into()).await {
                            error!("Error marking task as failed: {}", e);
                        }
                    } else {
                        if let Err(e) = queue_service.mark_task_completed(task.id.clone()).await {
                            error!("Error marking task as completed: {}", e);
                        }
                        info!("Task {} completed. Found {} total products", task.id, all_products.len());
                    }
                });
                
                running_tasks.push(handle);
            } else {
                break;
            }
        }

        // Wait a bit before checking for new tasks
        sleep(Duration::from_secs(1)).await;
    }
}

async fn scrape_store_dyn(
    store: Store,
    scraper: &dyn ScraperService,
    query: &str,
) -> Result<Vec<rust_scraper::models::product::Product>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(scraper.search_products(query).await?)
} 