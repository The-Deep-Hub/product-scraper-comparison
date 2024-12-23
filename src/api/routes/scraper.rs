use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use std::{fmt, str::FromStr};

use crate::{
    error::AppResult,
    services::{
        cache::CacheService,
        queue::QueueService,
    },
    db::{
        repositories::scraping::ScrapingRepository,
        models::scraping::ScrapingTask,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Store {
    Bricodepot,
    Bauhaus,
    Leroy,
    Obramat,
}

impl fmt::Display for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Store::Bricodepot => write!(f, "bricodepot"),
            Store::Bauhaus => write!(f, "bauhaus"),
            Store::Leroy => write!(f, "leroy"),
            Store::Obramat => write!(f, "obramat"),
        }
    }
}

impl FromStr for Store {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bricodepot" => Ok(Store::Bricodepot),
            "bauhaus" => Ok(Store::Bauhaus),
            "leroy" => Ok(Store::Leroy),
            "obramat" => Ok(Store::Obramat),
            _ => Err(format!("Invalid store: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SearchRequest {
    #[validate(length(min = 1, max = 100))]
    pub query: String,
    pub stores: Option<Vec<Store>>,
    #[validate(range(min = 1, max = 100))]
    pub num_products: Option<i32>,
}

async fn search_products(
    request: web::Json<SearchRequest>,
    repo: web::Data<ScrapingRepository>,
    cache: web::Data<dyn CacheService>,
    queue: web::Data<dyn QueueService>,
) -> AppResult<HttpResponse> {
    request.validate()?;

    // Create a longer-lived default value
    let default_stores = vec![Store::Bricodepot];
    let stores = request.stores.as_ref().unwrap_or(&default_stores);
    let store_list = stores.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",");
    
    // Check cache first
    let cache_key = format!("search:{}:{}", request.query, store_list);
    if let Some(cached_results) = cache.get(&cache_key).await? {
        // Track this as a popular search
        cache.get_popular_searches(10).await?;
        let parsed_results: serde_json::Value = serde_json::from_str(&cached_results)?;
        let response = serde_json::json!({
            "results": parsed_results,
            "cache_status": "hit",
            "stores": store_list
        });
        return Ok(HttpResponse::Ok().json(response));
    }

    // If not in cache, create new task
    let task_id = Uuid::new_v4();
    let _tasks = queue.create_scraping_tasks(&request, task_id).await?;

    for store in stores {
        let task = ScrapingTask {
            id: None,
            task_id,
            query: request.query.clone(),
            store: store.to_string(),
            status: "pending".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.create_task(&task).await?;
    }

    let response = serde_json::json!({
        "task_id": task_id,
        "message": "Search request accepted",
        "cache_status": "miss",
        "stores": store_list
    });
    Ok(HttpResponse::Accepted().json(response))
}

async fn get_search_results(
    path: web::Path<Uuid>,
    repo: web::Data<ScrapingRepository>,
) -> AppResult<HttpResponse> {
    let task_id = path.into_inner();
    let task = repo.find_task_by_uuid(task_id).await?
        .ok_or_else(|| crate::error::AppError::TaskNotFound(task_id.to_string()))?;

    let results = repo.find_results_by_task(task_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "task": task,
        "results": results
    })))
}

async fn get_active_tasks(
    repo: web::Data<ScrapingRepository>,
) -> AppResult<HttpResponse> {
    let tasks = repo.find_active_tasks().await?;
    Ok(HttpResponse::Ok().json(tasks))
}

async fn cancel_search(
    path: web::Path<Uuid>,
    repo: web::Data<ScrapingRepository>,
    queue: web::Data<dyn QueueService>,
) -> AppResult<HttpResponse> {
    let task_id = path.into_inner();
    
    queue.cancel_task(task_id).await?;
    repo.update_task_status(task_id, "cancelled").await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Search cancelled successfully"
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/scraper")
            .route("", web::post().to(search_products))
            .route("/tasks", web::get().to(get_active_tasks))
            .route("/{id}", web::get().to(get_search_results))
            .route("/{id}/cancel", web::post().to(cancel_search)),
    );
} 