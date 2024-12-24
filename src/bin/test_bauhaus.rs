use rust_scraper::{
    clients::zyte::ZyteClient,
    scrapers::BauhausScraper,
    services::scraper::ScraperService,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with more detailed output
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,test_bauhaus=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Create Zyte client
    let client = ZyteClient::new()?;
    
    // Create Bauhaus scraper
    let scraper = BauhausScraper::new(client);

    // Test search query
    let query = "martillo";
    info!("Searching for '{}'...", query);

    // Perform scraping
    match scraper.search_products(query).await {
        Ok(products) => {
            println!("\nFound {} products:", products.len());
            for (i, product) in products.iter().enumerate() {
                println!("\nProduct {}:", i + 1);
                println!("Name: {}", product.name);
                println!("URL: {}", product.url);
                if let Some(price) = &product.price {
                    println!("Price: {} {}", price.amount, price.currency);
                }
                if let Some(desc) = &product.description {
                    println!("Description: {}", desc);
                }
                if let Some(img) = &product.image_url {
                    println!("Image URL: {}", img);
                }
                if let Some(metadata) = &product.metadata {
                    println!("\nMetadata:");
                    for (key, value) in metadata {
                        println!("  {}: {}", key, value);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error during scraping: {}", e);
        }
    }

    Ok(())
} 