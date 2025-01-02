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
            let queue = Arc::clone(&self.queue_port);
            let event_publisher = Arc::clone(&self.event_publisher);

            tokio::spawn(async move {
                let start = Instant::now();
                info!("Processing query '{}' for store {}", query, store);

                // Find the appropriate scraper
                let scraper = scrapers.iter()
                    .find(|s| s.get_store() == store)
                    .expect("Scraper not found for store");

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

                        // Publish ProductsScraped event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::ProductsScraped {
                                metadata: EventMetadata::new(),
                                query: query.clone(),
                                products: products.clone(),
                            })
                            .await
                        {
                            error!("Failed to publish ProductsScraped event: {}", e);
                        }

                        // Publish ScrapeJobCompleted event
                        if let Err(e) = event_publisher
                            .publish(DomainEvent::ScrapeJobCompleted {
                                metadata: EventMetadata::new(),
                                query,
                                products,
                            })
                            .await
                        {
                            error!("Failed to publish ScrapeJobCompleted event: {}", e);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Error scraping products for query '{}' in store {}: {}",
                            query, store, e
                        );
                    }
                }
            });
        }

        Ok(())
    }
} 