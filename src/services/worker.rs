use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::error;

use crate::{
    error::{AppError, AppResult},
    services::{
        queue::{QueueService, RabbitMQQueue},
        scraper::ScraperService,
    },
};

impl From<tokio::task::JoinError> for AppError {
    fn from(err: tokio::task::JoinError) -> Self {
        AppError::InternalServerError(format!("Task join error: {}", err))
    }
}

#[derive(Clone)]
pub struct WorkerService {
    queue: RabbitMQQueue,
    scraper: Arc<dyn ScraperService>,
    poll_interval: Duration,
    max_concurrent_jobs: usize,
}

impl WorkerService {
    pub fn new(
        queue: RabbitMQQueue,
        scraper: Arc<dyn ScraperService>,
        poll_interval: Duration,
        max_concurrent_jobs: usize,
    ) -> Self {
        Self {
            queue,
            scraper,
            poll_interval,
            max_concurrent_jobs,
        }
    }

    pub async fn start(&self) -> AppResult<()> {
        loop {
            let tasks = self.queue.get_pending_tasks(self.max_concurrent_jobs).await?;
            
            if tasks.is_empty() {
                sleep(self.poll_interval).await;
                continue;
            }

            for task in tasks {
                let scraper = Arc::clone(&self.scraper);
                let queue = self.queue.clone();
                
                let handle = tokio::spawn(async move {
                    match scraper.get_product_details(&task.url).await {
                        Ok(_) => {
                            if let Err(e) = queue.mark_task_completed(&task.id).await {
                                error!("Failed to mark task as completed: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to process task {}: {}", task.id, e);
                            if let Err(e) = queue.mark_task_failed(&task.id).await {
                                error!("Failed to mark task as failed: {}", e);
                            }
                        }
                    }
                });

                handle.await.map_err(|e| {
                    error!("Task panicked: {}", e);
                    e
                })?;
            }

            sleep(self.poll_interval).await;
        }
    }
} 