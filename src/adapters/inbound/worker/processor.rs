use crate::domain::{
    ports::outbound::{QueuePort, ScraperPort, EventPublisherPort},
    events::{DomainEvent, EventMetadata},
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::{info, error};

const DEFAULT_LIMIT: Option<usize> = Some(100); // Default limit for product scraping
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const CHANNEL_BUFFER_SIZE: usize = 100;

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
        while let Some((query, store)) = rx.recv().await {
            let scrapers = Arc::clone(&self.scrapers);
            let event_publisher = Arc::clone(&self.event_publisher);

            tokio::spawn(async move {
                let start = Instant::now();
                
                // Emit JobStarted event
                if let Err(e) = event_publisher
                    .publish(DomainEvent::JobStarted {
                        metadata: EventMetadata::new(),
                        query: query.clone(),
                        store: store.clone(),
                    })
                    .await
                {
                    error!("Failed to publish JobStarted event: {}", e);
                }

                // Find the appropriate scraper
                let scraper = match scrapers.iter().find(|s| s.get_store() == store) {
                    Some(s) => s,
                    None => {
                        error!("Scraper not found for store {}", store);
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::JobFailed {
                                metadata: EventMetadata::new(),
                                query,
                                store,
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
                match scraper.scrape_products(&query, DEFAULT_LIMIT).await {
                    Ok(products) => {
                        info!(
                            "Found {} products for query '{}' in store {} (took {:?})",
                            products.len(),
                            query,
                            store,
                            start.elapsed()
                        );

                        // Emit ProductsScraped event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::ProductsScraped {
                                metadata: EventMetadata::new(),
                                query: query.clone(),
                                products: products.clone(),
                                store: store.clone(),
                            })
                            .await
                        {
                            error!("Failed to publish ProductsScraped event: {}", e);
                        }

                        // Emit JobCompleted event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::JobCompleted {
                                metadata: EventMetadata::new(),
                                query,
                                store,
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
                            query, store, e
                        );
                        
                        // Emit JobFailed event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::JobFailed {
                                metadata: EventMetadata::new(),
                                query,
                                store,
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