use std::sync::Arc;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppResult, AppError},
    models::{product::Product, store::Store},
    services::{
        cache::CacheService,
        queue::{QueueService, TaskStatus},
        scraper::ScraperService,
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
}

#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    status: String,
    products: Option<Vec<Product>>,
    error: Option<String>,
}

#[post("/search")]
pub async fn search_products(
    request: web::Json<SearchRequest>,
    cache_service: web::Data<Arc<dyn CacheService>>,
    queue_service: web::Data<Arc<dyn QueueService>>,
) -> AppResult<impl Responder> {
    tracing::info!("Received search request: {:?}", request);
    
    // Try to get from cache first
    match cache_service.get_search_results(&request.query).await {
        Ok(Some(products)) => {
            tracing::info!("Cache hit for query: {}", request.query);
            return Ok(HttpResponse::Ok().json(products));
        }
        Ok(None) => {
            tracing::info!("Cache miss for query: {}", request.query);
        }
        Err(e) => {
            tracing::error!("Cache error: {}", e);
            return Err(AppError::InternalServerError(format!("Cache error: {}", e)));
        }
    }
    
    // If not in cache, create and wait for task
    match queue_service.create_task(request.query.clone()).await {
        Ok(task_id) => {
            tracing::info!("Created task {} for query: {}", task_id, request.query);
            
            // Wait for task completion with timeout
            let mut attempts = 0;
            const MAX_ATTEMPTS: u32 = 30; // 30 seconds timeout
            
            while attempts < MAX_ATTEMPTS {
                match queue_service.get_task(&task_id).await {
                    Ok(task) => match task.status {
                        TaskStatus::Completed => {
                            // Task completed, get results from cache
                            match cache_service.get_search_results(&request.query).await {
                                Ok(Some(products)) => {
                                    return Ok(HttpResponse::Ok().json(products));
                                }
                                _ => {
                                    return Err(AppError::InternalServerError(
                                        "Task completed but results not found in cache".to_string()
                                    ));
                                }
                            }
                        }
                        TaskStatus::Failed => {
                            return Err(AppError::InternalServerError(
                                task.error.unwrap_or_else(|| "Task failed".to_string())
                            ));
                        }
                        _ => {
                            // Task still processing, wait and retry
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            attempts += 1;
                            continue;
                        }
                    },
                    Err(e) => {
                        return Err(AppError::InternalServerError(format!("Failed to check task status: {}", e)));
                    }
                }
            }
            
            // Timeout reached
            Err(AppError::InternalServerError("Task processing timeout".to_string()))
        }
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            Err(AppError::InternalServerError(format!("Failed to create task: {}", e)))
        }
    }
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
        log::error!("Failed to cache product details: {}", e);
    }
    
    Ok(HttpResponse::Ok().json(product))
}

#[get("/task/{task_id}")]
pub async fn get_task_status(
    task_id: web::Path<String>,
    queue_service: web::Data<Arc<dyn QueueService>>,
    cache_service: web::Data<Arc<dyn CacheService>>,
) -> AppResult<impl Responder> {
    let task = queue_service.get_task(&task_id).await?;
    
    let response = match task.status {
        TaskStatus::Completed => {
            let products = cache_service.get_search_results(&task.query).await?;
            TaskStatusResponse {
                status: task.status.to_string(),
                products,
                error: None,
            }
        }
        TaskStatus::Failed => TaskStatusResponse {
            status: task.status.to_string(),
            products: None,
            error: task.error,
        },
        _ => TaskStatusResponse {
            status: task.status.to_string(),
            products: None,
            error: None,
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
} 