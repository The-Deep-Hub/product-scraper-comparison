use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppResult, AppError},
    services::queue::QueueService,
    db::repositories::scraping::ScrapingRepository,
    api::routes::scraper::SearchRequest,
};

#[derive(Debug, Deserialize)]
pub struct TaskQuery {
    pub status: Option<String>,
    pub store: Option<String>,
}

async fn get_tasks(
    _query: web::Query<TaskQuery>,
    repo: web::Data<ScrapingRepository>,
) -> AppResult<HttpResponse> {
    let tasks = repo.find_active_tasks().await?;
    Ok(HttpResponse::Ok().json(tasks))
}

async fn retry_task(
    path: web::Path<Uuid>,
    repo: web::Data<ScrapingRepository>,
    queue: web::Data<dyn QueueService>,
) -> AppResult<HttpResponse> {
    let task_id = path.into_inner();
    let task = repo.find_task_by_uuid(task_id).await?
        .ok_or_else(|| crate::error::AppError::TaskNotFound(task_id.to_string()))?;

    // Create a new search request from the task
    let request = SearchRequest {
        query: task.query.clone(),
        store: task.store,
    };

    // Publish the task to the queue
    queue.get_ref().enqueue_product_details_task(&request.query).await?;

    // Update task status
    repo.update_task_status(task_id, "pending").await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Task retry initiated"
    })))
}

pub async fn get_job_status(
    path: web::Path<String>,
    _query: web::Query<TaskQuery>,
    repo: web::Data<ScrapingRepository>,
) -> AppResult<HttpResponse> {
    let task_id = uuid::Uuid::parse_str(&path).unwrap();
    let task = repo.find_task_by_uuid(task_id).await?
        .ok_or_else(|| AppError::TaskNotFound(task_id.to_string()))?;

    Ok(HttpResponse::Ok().json(task))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/jobs")
            .route("", web::get().to(get_tasks))
            .route("/{id}/retry", web::post().to(retry_task)),
    );
} 