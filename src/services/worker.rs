use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

use crate::error::AppResult;
use crate::services::queue::QueueService;
use crate::services::scraper::ScraperService;

pub struct WorkerService {
    queue_service: Arc<dyn QueueService>,
    scraper_service: Arc<dyn ScraperService>,
    polling_interval: Duration,
    max_concurrent_tasks: usize,
}

impl WorkerService {
    pub fn new(
        queue_service: Arc<dyn QueueService>,
        scraper_service: Arc<dyn ScraperService>,
        polling_interval: Duration,
        max_concurrent_tasks: usize,
    ) -> Self {
        Self {
            queue_service,
            scraper_service,
            polling_interval,
            max_concurrent_tasks,
        }
    }

    pub async fn start(&self) -> AppResult<()> {
        info!("Starting worker service");
        
        loop {
            let tasks = self.queue_service.get_pending_tasks(self.max_concurrent_tasks).await?;
            
            if tasks.is_empty() {
                sleep(self.polling_interval).await;
                continue;
            }

            let mut handles = Vec::new();

            for task in tasks {
                let scraper_service = Arc::clone(&self.scraper_service);
                let queue_service = Arc::clone(&self.queue_service);
                
                let handle = tokio::spawn(async move {
                    match scraper_service.get_product_details(&task.url).await {
                        Ok(_) => {
                            if let Err(e) = queue_service.mark_task_completed(&task.id).await {
                                error!("Failed to mark task as completed: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to process task {}: {}", task.id, e);
                            if let Err(e) = queue_service.mark_task_failed(&task.id).await {
                                error!("Failed to mark task as failed: {}", e);
                            }
                        }
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                if let Err(e) = handle.await {
                    error!("Task panicked: {}", e);
                }
            }

            sleep(self.polling_interval).await;
        }
    }
} 