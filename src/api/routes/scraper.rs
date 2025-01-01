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
    
    // Try to get from store-specific caches first
    let mut all_cached_products = Vec::new();
    let mut missing_stores = Vec::new();
    
    for store_name in &request.stores {
        if let Some(store) = Store::from_str(store_name) {
            match app_state.cache_port.get_store_products(&request.query, &store).await {
                Ok(products) if !products.is_empty() => {
                    info!("Cache hit for store {} with {} products", store_name, products.len());
                    all_cached_products.extend(products);
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
    
    // If we have all products from cache, return them
    if missing_stores.is_empty() && !all_cached_products.is_empty() {
        info!("All stores found in cache, returning {} products", all_cached_products.len());
        let mut stores: HashMap<String, Vec<Product>> = HashMap::new();
        for product in all_cached_products {
            stores
                .entry(product.store().to_string().to_lowercase())
                .or_insert_with(Vec::new)
                .push(product);
        }
        
        return Ok(HttpResponse::Ok().json(TaskStatusResponse {
            status: "completed".to_string(),
            stores: Some(stores),
            error: None,
        }));
    }
    
    // Create tasks only for stores that weren't in cache
    let mut task_ids = Vec::new();
    for (store_name, store) in missing_stores {
        match app_state.queue_port.enqueue_store_scrape_job(&request.query, &store).await {
            Ok(()) => {
                let task_id = uuid::Uuid::new_v4().to_string();
                task_ids.push(task_id.clone());
                
                // Store task ID to query mapping in Redis directly
                let task_key = format!("task_query:{}", task_id);
                let task_data = format!("{}:{}", store_name, request.query);
                if let Err(e) = app_state.cache_port.set_value(&task_key, &task_data, Some(300)).await {
                    error!("Failed to store task query mapping: {}", e);
                }
                
                info!("Created task {} for store {} and query: {}", task_id, store_name, request.query);
            },
            Err(e) => {
                error!("Failed to create search task for store {}: {}", store_name, e);
                return Err(AppError::InternalServerError(format!("Failed to create search task for store {}: {}", store_name, e)));
            }
        }
    }
    
    let main_task_id = task_ids.first().unwrap_or(&uuid::Uuid::new_v4().to_string()).clone();
    Ok(HttpResponse::Accepted().json(SearchResponse {
        task_id: main_task_id,
        status: "processing".to_string(),
        message: format!("Created {} tasks for processing. Check status using the task ID.", task_ids.len()),
    }))
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
            let (store_name, query) = (parts[0], parts[1]);
            
            // Parse store
            let store = Store::from_str(store_name);
            
            // Try to get products from store-specific cache first
            let products = if let Some(store) = store {
                match app_state.cache_port.get_store_products(query, &store).await {
                    Ok(products) if !products.is_empty() => products,
                    _ => {
                        // If no store-specific results, try all products cache
                        match app_state.cache_port.get_products(query).await {
                            Ok(all_products) => all_products.into_iter()
                                .filter(|p| p.store().to_string().to_lowercase() == store_name.to_lowercase())
                                .collect(),
                            Err(_) => Vec::new(),
                        }
                    }
                }
            } else {
                // If store is invalid, try all products cache
                app_state.cache_port.get_products(query).await.unwrap_or_default()
            };
            
            if !products.is_empty() {
                // Group products by store
                let mut stores: HashMap<String, Vec<Product>> = HashMap::new();
                for product in products {
                    stores
                        .entry(product.store().to_string().to_lowercase())
                        .or_insert_with(Vec::new)
                        .push(product);
                }
                
                Ok(HttpResponse::Ok().json(TaskStatusResponse {
                    status: "completed".to_string(),
                    stores: Some(stores),
                    error: None,
                }))
            } else {
                Ok(HttpResponse::Ok().json(TaskStatusResponse {
                    status: "processing".to_string(),
                    stores: None,
                    error: None,
                }))
            }
        },
        Ok(None) => Ok(HttpResponse::Ok().json(TaskStatusResponse {
            status: "not_found".to_string(),
            stores: None,
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