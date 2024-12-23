use actix_web::{web, HttpResponse};
use csv::Writer;
use serde::Serialize;

use crate::{
    error::AppResult,
    db::repositories::scraping::ScrapingRepository,
};

#[derive(Serialize)]
struct ResultRow {
    store: String,
    title: String,
    price: f64,
    url: String,
}

pub async fn export_results(
    path: web::Path<String>,
    repo: web::Data<ScrapingRepository>,
) -> AppResult<HttpResponse> {
    let task_id = uuid::Uuid::parse_str(&path).unwrap();
    let results = repo.find_results_by_task(task_id).await?;

    let mut wtr = Writer::from_writer(vec![]);
    for result in results {
        wtr.serialize(ResultRow {
            store: result.store,
            title: result.product_name,
            price: result.price,
            url: result.url,
        })?;
    }

    let csv_data = String::from_utf8(wtr.into_inner()?)?;
    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .body(csv_data))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/results")
            .route("/{id}/export", web::get().to(export_results))
    );
} 