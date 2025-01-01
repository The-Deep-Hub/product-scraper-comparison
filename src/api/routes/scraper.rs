use std::sync::Arc;
use std::collections::HashMap;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use tracing::{info, error};

use crate::{
    error::{AppResult, AppError},
    domain::models::product::Product,
    domain::models::store::Store,
    core::app_state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    query: String,
    stores: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    task_id: String,
    status: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    status: String,
    stores: Option<HashMap<String, Vec<Product>>>,
    pending_stores: Option<Vec<String>>,
    error: Option<String>,
}

const TASK_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[post("/search")]
pub async fn search_products(
    request: web::Json<SearchRequest>,
    app_state: web::Data<AppState>,
) -> AppResult<impl Responder> {
    info!("Received search request: {:?}", request);
    
    // Validate request
    if request.query.trim().is_empty() {
        error!("Empty search query received");
        return Err(AppError::BadRequest("Search query cannot be empty".into()));
    }
    
    if request.stores.is_empty() {
        error!("No stores specified");
        return Err(AppError::BadRequest("At least one store must be specified".into()));
    }
    
    // Handle "all" stores case
    let stores = if request.stores.len() == 1 && request.stores[0].to_lowercase() == "all" {
        info!("All stores requested");
        vec!["Leroy".to_string(), "Bauhaus".to_string(), "Bricodepot".to_string()]
    } else {
        request.stores.clone()
    };
    
    // Try to get from store-specific caches first
    let mut cached_products = Vec::new();
    let mut missing_stores = Vec::new();
    let mut found_stores = Vec::new();
    
    for store_name in &stores {
        if let Some(store) = Store::from_str(store_name) {
            match app_state.cache_port.get_store_products(&request.query, &store).await {
                Ok(products) if !products.is_empty() => {
                    info!("Cache hit for store {} with {} products", store_name, products.len());
                    cached_products.extend(products);
                    found_stores.push(store_name.clone());
                },
                _ => {
                    info!("Cache miss for store {}", store_name);
                    missing_stores.push((store_name, store));
                }
            }
        } else {
            error!("Invalid store name: {}", store_name);
            return Err(AppError::BadRequest(format!("Invalid store name: {}", store_name)));
        }
    }
    
    // If we have all products from cache, return them immediately
    if missing_stores.is_empty() {
        info!("All stores found in cache, returning {} products", cached_products.len());
        let mut stores: HashMap<String, Vec<Product>> = HashMap::new();
        for product in cached_products {
            stores
                .entry(product.store().to_string().to_lowercase())
                .or_insert_with(Vec::new)
                .push(product);
        }
        
        return Ok(HttpResponse::Ok().json(TaskStatusResponse {
            status: "completed".to_string(),
            stores: Some(stores),
            pending_stores: None,
            error: None,
        }));
    }
    
    // Create tasks only for stores that weren't in cache
    let mut task_ids = Vec::new();
    let main_task_id = uuid::Uuid::new_v4().to_string();
    
    // Log which stores were found in cache and which need scraping
    if !found_stores.is_empty() {
        info!(
            "Using cached results for stores: [{}], scraping for: [{}]",
            found_stores.join(", "),
            missing_stores.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(", ")
        );
    }
    
    // Store all stores and query mapping for the main task
    let task_key = format!("task_query:{}", main_task_id);
    let stores_list = missing_stores.iter()
        .map(|(name, _)| name.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let task_data = format!("{}:{}", stores_list, request.query);
    if let Err(e) = app_state.cache_port.set_value(&task_key, &task_data, Some(300)).await {
        error!("Failed to store task query mapping: {}", e);
    }
    
    for (store_name, store) in missing_stores {
        match app_state.queue_port.enqueue_store_scrape_job(&request.query, &store).await {
            Ok(()) => {
                let task_id = uuid::Uuid::new_v4().to_string();
                task_ids.push(task_id.clone());
                
                // Store subtask mapping
                let subtask_key = format!("subtask:{}:{}", main_task_id, store_name);
                let subtask_data = format!("{}:{}", store_name, request.query);
                if let Err(e) = app_state.cache_port.set_value(&subtask_key, &subtask_data, Some(300)).await {
                    error!("Failed to store subtask mapping: {}", e);
                }
                
                info!("Created subtask {} for store {} and query: {}", task_id, store_name, request.query);
            },
            Err(e) => {
                error!("Failed to create search task for store {}: {}", store_name, e);
                return Err(AppError::InternalServerError(format!("Failed to create search task for store {}: {}", store_name, e)));
            }
        }
    }
    
    // If we have some cached results and some pending tasks
    if !cached_products.is_empty() {
        let mut stores: HashMap<String, Vec<Product>> = HashMap::new();
        for product in cached_products {
            stores
                .entry(product.store().to_string().to_lowercase())
                .or_insert_with(Vec::new)
                .push(product);
        }
        
        Ok(HttpResponse::Accepted().json(SearchResponse {
            task_id: main_task_id,
            status: "partial_content".to_string(),
            message: format!(
                "Found cached results for [{}]. Created {} tasks for remaining stores. Check status using the task ID.",
                found_stores.join(", "),
                task_ids.len()
            ),
        }))
    } else {
        // If all stores need to be scraped
        Ok(HttpResponse::Accepted().json(SearchResponse {
            task_id: main_task_id,
            status: "processing".to_string(),
            message: format!("Created {} tasks for processing. Check status using the task ID.", task_ids.len()),
        }))
    }
}

#[get("/task/{task_id}")]
pub async fn get_task_status(
    task_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> AppResult<impl Responder> {
    // Get the query from the task ID mapping
    let task_key = format!("task_query:{}", task_id.as_str());
    match app_state.cache_port.get_value(&task_key).await {
        Ok(Some(task_data)) => {
            let parts: Vec<&str> = task_data.split(':').collect();
            if parts.len() != 2 {
                return Err(AppError::InternalServerError("Invalid task data format".into()));
            }
            let (stores_list, query) = (parts[0], parts[1]);
            
            // Split stores list into individual stores
            let store_names: Vec<&str> = stores_list.split(',').collect();
            let mut available_stores = HashMap::new();
            let mut pending_stores = Vec::new();
            
            // Get products for each store
            for store_name in store_names {
                if let Some(store) = Store::from_str(store_name) {
                    match app_state.cache_port.get_store_products(query, &store).await {
                        Ok(products) if !products.is_empty() => {
                            info!("Found {} products for store {} and query {}", products.len(), store_name, query);
                            available_stores.insert(store.to_string().to_lowercase(), products);
                        },
                        _ => {
                            info!("Store {} still processing for query {}", store_name, query);
                            pending_stores.push(store_name.to_string());
                        }
                    }
                }
            }
            
            let status = if pending_stores.is_empty() {
                "completed".to_string()
            } else {
                "processing".to_string()
            };
            
            let pending_stores = if pending_stores.is_empty() {
                None
            } else {
                Some(pending_stores)
            };
            
            let stores = if available_stores.is_empty() {
                None
            } else {
                Some(available_stores)
            };
            
            Ok(HttpResponse::Ok().json(TaskStatusResponse {
                status,
                stores,
                pending_stores,
                error: None,
            }))
        },
        Ok(None) => Ok(HttpResponse::Ok().json(TaskStatusResponse {
            status: "not_found".to_string(),
            stores: None,
            pending_stores: None,
            error: Some("Task not found".into()),
        })),
        Err(e) => {
            error!("Failed to get task query mapping: {}", e);
            Err(AppError::InternalServerError(format!("Failed to get task query mapping: {}", e)))
        }
    }
}

#[get("/product/{url}")]
pub async fn get_product_details(
    url: web::Path<String>,
    app_state: web::Data<AppState>,
) -> AppResult<impl Responder> {
    // Try to get from cache first
    match app_state.cache_port.get_products(&url).await {
        Ok(cached_products) if !cached_products.is_empty() => {
            Ok(HttpResponse::Ok().json(&cached_products[0]))
        },
        Ok(_) | Err(_) => {
            // If not in cache, scrape it
            match app_state.scraper_service.get_product_details(&url).await {
                Ok(product) => {
                    // Cache the result
                    if let Err(e) = app_state.cache_port.cache_products(&url, &[product.clone()]).await {
                        error!("Failed to cache product details: {}", e);
                    }
                    Ok(HttpResponse::Ok().json(product))
                },
                Err(e) => {
                    error!("Failed to get product details: {}", e);
                    Err(AppError::InternalServerError(format!("Failed to get product details: {}", e)))
                }
            }
        }
    }
}