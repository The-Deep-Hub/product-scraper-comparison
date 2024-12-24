use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

use crate::{
    error::AppResult,
    services::{
        queue::QueueService,
        scraper::ScraperService,
    },
};

pub struct WorkerService {
    queue_service: Arc<dyn QueueService>,
    scraper_service: Arc<dyn ScraperService>,
}

impl WorkerService {
    pub fn new(
        queue_service: Arc<dyn QueueService>,
        scraper_service: Arc<dyn ScraperService>,
    ) -> Self {
        Self {
            queue_service,
            scraper_service,
        }
    }

    pub async fn run(&self) -> AppResult<()> {
        info!("Starting worker service...");
        
        loop {
            // Get pending tasks from queue
            if let Some(task) = self.queue_service.get_pending_task().await? {
                info!("Processing task: {}", task.id);
                
                // Process the task
                match self.scraper_service.search_products(&task.query).await {
                    Ok(products) => {
                        info!("Found {} products for query: {}", products.len(), task.query);
                        self.queue_service.mark_task_completed(task.id).await?;
                    }
                    Err(e) => {
                        error!("Failed to process task {}: {}", task.id, e);
                        self.queue_service.mark_task_failed(task.id, e.to_string()).await?;
                    }
                }
            }
            
            // Sleep for a short duration before checking for new tasks
            sleep(Duration::from_secs(1)).await;
        }
    }
} 