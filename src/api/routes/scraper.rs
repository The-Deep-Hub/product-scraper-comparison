use std::sync::Arc;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppResult,
    models::{product::Product, store::Store},
    services::{
        cache::CacheService,
        queue::QueueService,
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
    scraper_service: web::Data<Arc<dyn ScraperService>>,
) -> AppResult<impl Responder> {
    let products = match request.store {
        Some(store) => {
            scraper_service.search_store_products(store, &request.query).await?
        }
        None => {
            scraper_service.search_products(&request.query).await?
        }
    };
    
    Ok(HttpResponse::Ok().json(products))
}

#[get("/product")]
pub async fn get_product_details(
    url: web::Query<String>,
    scraper_service: web::Data<Arc<dyn ScraperService>>,
) -> AppResult<impl Responder> {
    let product = scraper_service.get_product_details(&url).await?;
    Ok(HttpResponse::Ok().json(product))
}

#[get("/task/{task_id}")]
pub async fn get_task_status(
    task_id: web::Path<String>,
    queue_service: web::Data<Arc<dyn QueueService>>,
    cache_service: web::Data<Arc<dyn CacheService>>,
) -> AppResult<impl Responder> {
    let task = queue_service.get_task(&task_id).await?;
    
    let response = match task.status.as_str() {
        "completed" => {
            let products = cache_service.get_search_results(&task.query).await?;
            TaskStatusResponse {
                status: task.status.to_string(),
                products,
                error: None,
            }
        }
        "failed" => TaskStatusResponse {
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