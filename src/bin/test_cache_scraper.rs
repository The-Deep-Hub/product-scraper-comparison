use std::sync;
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
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,test_cache_scraper=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize Redis
    let redis_url = format!(
        "redis://{}:{}@{}:{}/",
        std::env::var("REDIS_USER").unwrap_or_else(|_| "default".to_string()),
        std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set"),
        std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
        std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string()),
    );
    let redis_password = std::env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set");
    info!("Connecting to Redis at: {}", redis_url.replace(&redis_password, "****"));
    
    // Initialize services
    let cache_service = Box::new(RedisCacheService::new(&redis_url).await?) as Box<dyn CacheService>;
    let queue_service = Box::new(RabbitMQQueue::new().await?) as Box<dyn QueueService>;

    // Initialize Zyte client
    let zyte_client = ZyteClient::new()?.clone();

    // Initialize individual scrapers
    let leroy_scraper = LeroyScraper::new(zyte_client.clone());
    let bauhaus_scraper = BauhausScraper::new(zyte_client.clone());
    let bricodepot_scraper = BricodepotScraper::new(zyte_client);

    // Create combined scraper service
    let scraper_service = CombinedScraperService::new(
        leroy_scraper,
        bauhaus_scraper,
        bricodepot_scraper,
        cache_service,
        queue_service,
    );

    // Test search functionality
    info!("Testing search for query: taladro");
    info!("Test 1: Searching all stores (first time, no cache)");
    let products = scraper_service.search_products("taladro").await?;
    info!("Found {} products across all stores", products.len());

    info!("Test 2: Searching again (should hit cache)");
    let cached_products = scraper_service.search_products("taladro").await?;
    info!("Found {} products from cache", cached_products.len());

    info!("Test 3: Testing store-specific search for Leroy Merlin");
    let leroy_products = scraper_service.search_store_products(Store::LeroyMerlin, "taladro").await?;
    info!("Found {} products from Leroy Merlin", leroy_products.len());

    info!("Test 4: Testing store-specific search for Bricodepot");
    let bricodepot_products = scraper_service.search_store_products(Store::Bricodepot, "taladro").await?;
    info!("Found {} products from Bricodepot", bricodepot_products.len());

    // Test 5: Product details
    if let Some(first_product) = leroy_products.first() {
        info!("Test 5: Getting product details for: {}", first_product.url);
        let product = scraper_service.get_product_details(&first_product.url).await?;
        info!("Successfully retrieved product: {}", product.name);

        // Test cached product details
        info!("Test 6: Getting same product details (should hit cache)");
        let cached_product = scraper_service.get_product_details(&first_product.url).await?;
        info!("Successfully retrieved cached product: {}", cached_product.name);
    }

    Ok(())
} 