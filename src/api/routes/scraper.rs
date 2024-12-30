use std::sync::Arc;
use std::collections::HashMap;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};
use tracing::{info, error};

use crate::{
    error::{AppResult, AppError},
    models::{
        product::Product,
        store::Store,
        task::{MainTask, TaskStatus},
    },
    services::{
        cache::CacheService,
        queue::QueueService,
        scraper::ScraperService,
        task_splitter::TaskSplitterService,
    },
};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    query: String,
    store: Option<Store>,
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
    cache_service: web::Data<Arc<dyn CacheService>>,
    _queue_service: web::Data<Arc<dyn QueueService>>,
    task_splitter: web::Data<Arc<dyn TaskSplitterService>>,
) -> AppResult<impl Responder> {
    info!("Received search request: {:?}", request);
    info!("Query: {}, Store: {:?}", request.query, request.store);
    
    // Validate request
    if request.query.trim().is_empty() {
        error!("Empty search query received");
        return Err(AppError::BadRequest("Search query cannot be empty".into()));
    }
    
    // Try to get from cache first
    if let Ok(Some(products)) = cache_service.get_search_results(&request.query).await {
        info!("Cache hit, grouping {} products by store", products.len());
        
        // Group products by store
        let mut stores: HashMap<String, Vec<Product>> = HashMap::new();
        for product in products {
            stores
                .entry(product.store.to_string().to_lowercase())
                .or_insert_with(Vec::new)
                .push(product);
        }
        
        return Ok(HttpResponse::Ok().json(TaskStatusResponse {
            status: "completed".to_string(),
            stores: Some(stores),
            error: None,
        }));
    }
    
    info!("Cache miss for query: {}", request.query);
    
    // Create main task
    let main_task = MainTask::new(request.query.clone(), request.store);
    info!("Created main task with ID: {}", main_task.id);
    
    if let Err(e) = cache_service.set_main_task(&main_task).await {
        error!("Failed to set main task in cache: {}", e);
        return Err(AppError::InternalServerError("Failed to create task".into()));
    }
    
    info!("Created main task {} for query: {}", main_task.id, main_task.query);
    
    // Split task into store-specific tasks
    if let Err(e) = task_splitter.split_task(&main_task).await {
        error!("Failed to split task: {}", e);
        return Err(AppError::InternalServerError("Failed to split task".into()));
    }
    
    info!("Split task {} into store-specific tasks", main_task.id);
    
    // Wait for task completion with timeout
    match timeout(TASK_TIMEOUT, async {
        loop {
            match cache_service.get_main_task(&main_task.id).await {
                Ok(Some(task)) => {
                    if task.is_complete() {
                        // Combine all products from store results
                        let mut stores: HashMap<String, Vec<Product>> = HashMap::new();
                        for (store, products) in task.store_results {
                            stores.entry(store.to_string().to_lowercase()).or_insert_with(Vec::new).extend(products);
                        }
                        
                        // Cache combined results
                        if !stores.is_empty() {
                            if let Err(e) = cache_service.set_search_results(&request.query, &stores.values().cloned().flatten().collect::<Vec<_>>()).await {
                                error!("Failed to cache combined results: {}", e);
                            }
                        }
                        
                        info!("Task completed, returning {} stores", stores.len());
                        return Ok(HttpResponse::Ok()
                            .content_type("application/json")
                            .json(TaskStatusResponse {
                                status: "completed".to_string(),
                                stores: Some(stores),
                                error: None,
                            }));
                    } else if task.status == TaskStatus::Failed {
                        error!("Task failed: {}", task.error.as_deref().unwrap_or("Unknown error"));
                        return Ok(HttpResponse::InternalServerError()
                            .content_type("application/json")
                            .json(TaskStatusResponse {
                                status: "failed".to_string(),
                                stores: None,
                                error: task.error,
                            }));
                    }
                    
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
                Ok(None) => {
                    error!("Task not found in cache");
                    return Ok(HttpResponse::NotFound()
                        .content_type("application/json")
                        .json(TaskStatusResponse {
                            status: "error".to_string(),
                            stores: None,
                            error: Some("Task not found".to_string()),
                        }));
                }
                Err(e) => {
                    error!("Failed to check task status: {}", e);
                    return Ok(HttpResponse::InternalServerError()
                        .content_type("application/json")
                        .json(TaskStatusResponse {
                            status: "error".to_string(),
                            stores: None,
                            error: Some(format!("Failed to check task status: {}", e)),
                        }));
                }
            }
        }
    }).await {
        Ok(result) => result,
        Err(_) => {
            // Timeout occurred - return a properly formatted JSON response
            info!("Task processing timeout for ID: {}", main_task.id);
            Ok(HttpResponse::Accepted()
                .content_type("application/json")
                .json(SearchResponse {
                    task_id: main_task.id.clone(),
                    status: "processing".to_string(),
                    message: format!("Task {} is still processing. Please check status later using the task ID.", main_task.id),
                }))
        }
    }
}

#[get("/task/{task_id}")]
pub async fn get_task_status(
    task_id: web::Path<String>,
    cache_service: web::Data<Arc<dyn CacheService>>,
) -> AppResult<impl Responder> {
    let task = cache_service.get_main_task(&task_id).await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;
    
    let response = if task.is_complete() {
        let mut stores: HashMap<String, Vec<Product>> = HashMap::new();
        for (store, products) in task.store_results {
            stores.entry(store.to_string().to_lowercase()).or_insert_with(Vec::new).extend(products);
        }
        
        TaskStatusResponse {
            status: "completed".to_string(),
            stores: Some(stores),
            error: None,
        }
    } else if task.status == TaskStatus::Failed {
        TaskStatusResponse {
            status: "failed".to_string(),
            stores: None,
            error: task.error,
        }
    } else {
        TaskStatusResponse {
            status: task.status.to_string().to_lowercase(),
            stores: None,
            error: None,
        }
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[get("/product")]
pub async fn get_product_details(
    url: web::Query<String>,
    scraper_service: web::Data<Arc<dyn ScraperService>>,
    cache_service: web::Data<Arc<dyn CacheService>>,
) -> AppResult<impl Responder> {
    // Try to get from cache first
    if let Ok(Some(product)) = cache_service.get_product_details(&url).await {
        return Ok(HttpResponse::Ok().json(product));
    }
    
    // If not in cache, get from scraper and cache it
    let product = scraper_service.get_product_details(&url).await?;
    if let Err(e) = cache_service.set_product_details(&product).await {
        error!("Failed to cache product details: {}", e);
    }
    
    Ok(HttpResponse::Ok().json(product))
}