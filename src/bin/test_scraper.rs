use rust_scraper::{
    clients::zyte::ZyteClient,
    scrapers::LeroyScraper,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with more detailed output
    tracing_subscriber::fmt()
        .with_env_filter("rust_scraper=debug,test_scraper=debug")
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Create Zyte client
    let client = ZyteClient::new()?;
    
    // Create Leroy scraper
    let scraper = LeroyScraper::new(client)
        .ok_or_else(|| "Failed to initialize Leroy scraper")?;

    // Test search query
    let query = "martillo";
    let num_products = 5;

    println!("Searching for '{}', fetching {} products...", query, num_products);

    // Perform scraping
    match scraper.scrape(query, num_products).await {
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