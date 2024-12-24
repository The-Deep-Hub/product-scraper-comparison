use actix_web::{
    web::{self, Data, Json, Path},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppResult,
    services::{
        queue::{QueueService, RabbitMQQueue},
        scraper::ScraperService,
    },
};

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub task_id: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/scraper")
            .route("", web::post().to(search_products))
            .route("/tasks/{task_id}", web::get().to(get_task_status))
            .route("/tasks/{task_id}", web::delete().to(cancel_task))
            .route("/results/{task_id}", web::get().to(get_task_results)),
    );
}

async fn search_products(
    request: Json<SearchRequest>,
    queue: Data<RabbitMQQueue>,
    scraper: Data<dyn ScraperService>,
) -> AppResult<HttpResponse> {
    let task_id = queue.get_ref().create_task(&request.query).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "task_id": task_id,
        "message": "Search request accepted"
    })))
}

async fn get_task_status(
    task_id: Path<String>,
    queue: Data<RabbitMQQueue>,
) -> AppResult<HttpResponse> {
    let tasks = queue.get_ref().get_pending_tasks(1).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": if tasks.is_empty() { "completed" } else { "pending" }
    })))
}

async fn get_task_results(
    _task_id: Path<String>,
    queue: Data<RabbitMQQueue>,
) -> AppResult<HttpResponse> {
    let tasks = queue.get_ref().get_pending_tasks(1).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "results": tasks
    })))
}

async fn cancel_task(
    task_id: Path<String>,
    queue: Data<RabbitMQQueue>,
) -> AppResult<HttpResponse> {
    queue.get_ref().mark_task_failed(&task_id).await?;
    Ok(HttpResponse::Ok().finish())
} 