use rust_scraper::{
    clients::zyte::ZyteClient,
    scrapers::LeroyScraper,
    services::scraper::ScraperService,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,test_scraper=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize Zyte client
    let client = ZyteClient::new()?;

    // Initialize scraper
    let scraper = LeroyScraper::new(client);

    // Test search
    let query = "taladro";
    info!("Searching for: {}", query);
    let products = scraper.search_products(query).await?;
    info!("Found {} products", products.len());

    for product in products {
        println!("{:#?}", product);
    }

    Ok(())
} 