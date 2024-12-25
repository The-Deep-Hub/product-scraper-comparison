use std::sync::Arc;
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
    products: Option<Vec<Product>>,
    error: Option<String>,
}

const TASK_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[post("/search")]
pub async fn search_products(
    request: web::Json<SearchRequest>,
    cache_service: web::Data<Arc<dyn CacheService>>,
    queue_service: web::Data<Arc<dyn QueueService>>,
    task_splitter: web::Data<Arc<dyn TaskSplitterService>>,
) -> AppResult<impl Responder> {
    info!("Received search request: {:?}", request);
    
    // Try to get from cache first
    match cache_service.get_search_results(&request.query).await {
        Ok(Some(products)) => {
            info!("Cache hit for query: {}", request.query);
            return Ok(HttpResponse::Ok().json(products));
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
    let main_task = MainTask::new(request.query.clone());
    cache_service.set_main_task(&main_task).await?;
    
    info!("Created main task {} for query: {}", main_task.id, main_task.query);
    
    // Split task into store-specific tasks
    task_splitter.split_task(&main_task).await?;
    
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
                        
                        return Ok(HttpResponse::Ok().json(all_products));
                    } else if task.status == TaskStatus::Failed {
                        return Err(AppError::InternalServerError(
                            task.error.unwrap_or_else(|| "Task failed".to_string())
                        ));
                    }
                    
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
                Ok(None) => {
                    return Err(AppError::InternalServerError("Task not found".to_string()));
                }
                Err(e) => {
                    return Err(AppError::InternalServerError(format!("Failed to check task status: {}", e)));
                }
            }
        }
    }).await {
        Ok(result) => result,
        Err(_) => {
            // Timeout occurred
            Ok(HttpResponse::Accepted().json(SearchResponse {
                task_id: main_task.id.clone(),
                status: "processing".to_string(),
                message: "Task is still processing. Please check status later using the task ID.".to_string(),
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