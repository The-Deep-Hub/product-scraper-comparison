use std::sync::Arc;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};

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
    status: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    status: String,
    products: Option<Vec<Product>>,
    error: Option<String>,
}

const TASK_TIMEOUT: Duration = Duration::from_secs(180);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

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
    
    // If not in cache, create task
    let task_id = match queue_service.create_task(request.query.clone()).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            return Err(AppError::InternalServerError(format!("Failed to create task: {}", e)));
        }
    };
    
    tracing::info!("Created task {} for query: {}", task_id, request.query);
    
    // Wait for task completion with timeout
    match timeout(TASK_TIMEOUT, async {
        loop {
            match queue_service.get_task(&task_id).await {
                Ok(task) => match task.status {
                    TaskStatus::Completed => {
                        // Task completed, get results from cache
                        match cache_service.get_search_results(&request.query).await {
                            Ok(Some(products)) => {
                                return Ok(HttpResponse::Ok().json(products));
                            }
                            Ok(None) => {
                                return Err(AppError::InternalServerError(
                                    "Task completed but results not found in cache".to_string()
                                ));
                            }
                            Err(e) => {
                                return Err(AppError::InternalServerError(format!("Cache error: {}", e)));
                            }
                        }
                    }
                    TaskStatus::Failed => {
                        return Err(AppError::InternalServerError(
                            task.error.unwrap_or_else(|| "Task failed".to_string())
                        ));
                    }
                    TaskStatus::Processing => {
                        tokio::time::sleep(POLL_INTERVAL).await;
                        continue;
                    }
                    TaskStatus::Pending => {
                        tokio::time::sleep(POLL_INTERVAL).await;
                        continue;
                    }
                },
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
                task_id: task_id.clone(),
                status: "processing".to_string(),
                message: "Task is still processing. Please check status later using the task ID.".to_string(),
            }))
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