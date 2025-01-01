fn main() {
    println!("Rust Scraper - A web scraping service");
    println!("\nAvailable commands:");
    println!("  cargo run --bin setup        Initialize required infrastructure (queues, etc.)");
    println!("  cargo run --bin api          Start the API server");
    println!("  cargo run --bin worker       Start the worker process");
    println!("\nFirst-time setup:");
    println!("1. Start infrastructure:        docker-compose up -d");
    println!("2. Run setup:                   cargo run --bin setup");
    println!("3. Start API:                   cargo run --bin api");
    println!("4. Start worker:                cargo run --bin worker");
}