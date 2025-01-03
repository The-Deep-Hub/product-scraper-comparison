use crate::domain::{
    ports::outbound::{QueuePort, ScraperPort, EventPublisherPort},
    events::{DomainEvent, EventMetadata},
    models::Store,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::{info, error};

const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const CHANNEL_BUFFER_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct ScrapeTask {
    pub query: String,
    pub store: Store,
    pub num_products: Option<usize>,
}

pub struct TaskProcessor {
    queue_port: Arc<dyn QueuePort>,
    scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl TaskProcessor {
    pub fn new(
        queue_port: Arc<dyn QueuePort>,
        scrapers: Arc<Vec<Arc<dyn ScraperPort>>>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            queue_port,
            scrapers,
            event_publisher,
        }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let (tx, mut rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
        let mut tasks = JoinSet::new();

        // Start consumers for each scraper
        for scraper in self.scrapers.iter() {
            let store = scraper.get_store();
            let queue = Arc::clone(&self.queue_port);
            let tx = tx.clone();

            tasks.spawn(async move {
                loop {
                    if let Err(e) = queue.consume_messages(&store, tx.clone()).await {
                        error!("Error consuming messages for {}: {}", store, e);
                        sleep(RECONNECT_DELAY).await;
                    }
                }
            });
        }

        // Process received messages
        while let Some(task) = rx.recv().await {
            let scrapers = Arc::clone(&self.scrapers);
            let event_publisher = Arc::clone(&self.event_publisher);

            tokio::spawn(async move {
                let start = Instant::now();
                
                // Emit JobStarted event
                if let Err(e) = event_publisher
                    .publish(DomainEvent::JobStarted {
                        metadata: EventMetadata::new(),
                        query: task.query.clone(),
                        store: task.store.clone(),
                    })
                    .await
                {
                    error!("Failed to publish JobStarted event: {}", e);
                }

                // Find the appropriate scraper
                let scraper = match scrapers.iter().find(|s| s.get_store() == task.store) {
                    Some(s) => s,
                    None => {
                        error!("Scraper not found for store {}", task.store);
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::JobFailed {
                                metadata: EventMetadata::new(),
                                query: task.query,
                                store: task.store,
                                error: "Scraper not found for store".to_string(),
                            })
                            .await
                        {
                            error!("Failed to publish JobFailed event: {}", e);
                        }
                        return;
                    }
                };

                // Scrape products
                match scraper.scrape_products(&task.query, task.num_products).await {
                    Ok(products) => {
                        info!(
                            "Found {} products for query '{}' in store {} (took {:?})",
                            products.len(),
                            task.query,
                            task.store,
                            start.elapsed()
                        );

                        // Emit ProductsScraped event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::ProductsScraped {
                                metadata: EventMetadata::new(),
                                query: task.query.clone(),
                                products: products.clone(),
                                store: task.store.clone(),
                            })
                            .await
                        {
                            error!("Failed to publish ProductsScraped event: {}", e);
                        }

                        // Emit JobCompleted event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::JobCompleted {
                                metadata: EventMetadata::new(),
                                query: task.query,
                                store: task.store,
                                products_count: products.len(),
                            })
                            .await
                        {
                            error!("Failed to publish JobCompleted event: {}", e);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Error scraping products for query '{}' in store {}: {}",
                            task.query, task.store, e
                        );
                        
                        // Emit JobFailed event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::JobFailed {
                                metadata: EventMetadata::new(),
                                query: task.query,
                                store: task.store,
                                error: e.to_string(),
                            })
                            .await
                        {
                            error!("Failed to publish JobFailed event: {}", e);
                        }
                    }
                }
            });
        }

        Ok(())
    }
} 