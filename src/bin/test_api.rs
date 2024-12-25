use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize HTTP client
    let client = Client::new();
    let base_url = "http://localhost:8080";

    println!("Testing search endpoint (first request, should create task)...");
    let search_response = client
        .post(&format!("{}/api/scraper/search", base_url))
        .json(&json!({
            "query": "martillo",
            "store": null
        }))
        .send()
        .await?;

    println!("Search Response Status: {}", search_response.status());
    let response_data = search_response.json::<serde_json::Value>().await?;
    println!("Search Response: {:#?}", response_data);

    // If we got a task_id, poll for results
    if let Some(task_id) = response_data.get("task_id").and_then(|v| v.as_str()) {
        println!("\nPolling task status...");
        loop {
            let status_response = client
                .get(&format!("{}/api/scraper/task/{}", base_url, task_id))
                .send()
                .await?;

            let status_data = status_response.json::<serde_json::Value>().await?;
            println!("Task Status: {:#?}", status_data);

            // Check if task is completed or failed
            if let Some(status) = status_data.get("status").and_then(|v| v.as_str()) {
                if status == "completed" || status == "failed" {
                    break;
                }
            }

            // Wait before polling again
            sleep(Duration::from_secs(2)).await;
        }
    }

    // Test the same search again (should hit cache)
    println!("\nTesting search endpoint again (should hit cache)...");
    let cached_response = client
        .post(&format!("{}/api/scraper/search", base_url))
        .json(&json!({
            "query": "martillo",
            "store": null
        }))
        .send()
        .await?;

    println!("Cached Response Status: {}", cached_response.status());
    let cached_data = cached_response.json::<serde_json::Value>().await?;
    println!("Cached Response: {:#?}", cached_data);

    Ok(())
} 