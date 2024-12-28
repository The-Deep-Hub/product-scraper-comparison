use std::sync::Arc;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};
use tracing::{info, error, debug};

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
    products: Option<Vec<Product>>,
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
    debug!("Received raw request: {:?}", request);
    info!("Processing search request - Query: {}, Store: {:?}", request.query, request.store);

    // Log that services are being used
    debug!("Attempting to use services for request processing...");
    
    // Validate request
    if request.query.trim().is_empty() {
        error!("Empty search query received");
        return Err(AppError::BadRequest("Search query cannot be empty".into()));
    }
    
    // Try to get from cache first
    debug!("Checking cache for query: {}", request.query);
    match cache_service.get_search_results(&request.query).await {
        Ok(Some(products)) => {
            info!("Cache hit for query: {}", request.query);
            info!("Returning {} products from cache", products.len());
            return Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(products));
        }
        Ok(None) => {
            info!("Cache miss for query: {}", request.query);
        }
        Err(e) => {
            error!("Cache error: {}", e);
            return Err(AppError::InternalServerError(format!("Cache error: {}", e)));
        }
    }
    
    // Create main task
    debug!("Creating main task for query: {}", request.query);
    let main_task = MainTask::new(request.query.clone(), request.store);
    info!("Created main task with ID: {}", main_task.id);
    
    if let Err(e) = cache_service.set_main_task(&main_task).await {
        error!("Failed to set main task in cache: {}", e);
        return Err(AppError::InternalServerError("Failed to create task".into()));
    }
    
    info!("Created main task {} for query: {}", main_task.id, main_task.query);
    
    // Split task into store-specific tasks
    debug!("Splitting task {} into store-specific tasks", main_task.id);
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
                        let mut all_products = Vec::new();
                        for products in task.store_results.values() {
                            all_products.extend(products.clone());
                        }
                        
                        // Cache combined results
                        if !all_products.is_empty() {
                            if let Err(e) = cache_service.set_search_results(&request.query, &all_products).await {
                                error!("Failed to cache combined results: {}", e);
                            }
                        }
                        
                        info!("Task completed, returning {} products", all_products.len());
                        return Ok(HttpResponse::Ok()
                            .content_type("application/json")
                            .json(TaskStatusResponse {
                                status: "completed".to_string(),
                                products: Some(all_products),
                                error: None,
                            }));
                    } else if task.status == TaskStatus::Failed {
                        error!("Task failed: {}", task.error.as_deref().unwrap_or("Unknown error"));
                        return Ok(HttpResponse::InternalServerError()
                            .content_type("application/json")
                            .json(TaskStatusResponse {
                                status: "failed".to_string(),
                                products: None,
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
                            products: None,
                            error: Some("Task not found".to_string()),
                        }));
                }
                Err(e) => {
                    error!("Failed to check task status: {}", e);
                    return Ok(HttpResponse::InternalServerError()
                        .content_type("application/json")
                        .json(TaskStatusResponse {
                            status: "error".to_string(),
                            products: None,
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
        let mut all_products = Vec::new();
        for products in task.store_results.values() {
            all_products.extend(products.clone());
        }
        
        TaskStatusResponse {
            status: "completed".to_string(),
            products: Some(all_products),
            error: None,
        }
    } else if task.status == TaskStatus::Failed {
        TaskStatusResponse {
            status: "failed".to_string(),
            products: None,
            error: task.error,
        }
    } else {
        TaskStatusResponse {
            status: task.status.to_string().to_lowercase(),
            products: None,
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