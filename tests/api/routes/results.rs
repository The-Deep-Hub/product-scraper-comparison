use actix_web::{test, web, App};
use mockall::predicate::*;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::routes::results::config,
    services::cache::MockCacheService,
    db::repositories::scraping::MockScrapingRepository,
};

#[actix_web::test]
async fn test_get_results_success() {
    // Create mocks
    let mut mock_cache = MockCacheService::new();
    let mut mock_repo = MockScrapingRepository::new();

    // Setup expectations
    mock_cache
        .expect_get_search_results()
        .returning(|_, _| Ok(None));

    mock_repo
        .expect_get_results_by_query()
        .returning(|_, _, _, _| Ok(vec![]));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri("/results?query=hammer&store=Bricodepot")
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn test_get_result_success() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_repo = MockScrapingRepository::new();
    let result_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_result()
        .with(eq(result_id.to_string()))
        .returning(|_| Ok(Some(/* create mock result */)));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/results/{}", result_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_result_not_found() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_repo = MockScrapingRepository::new();
    let result_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_result()
        .with(eq(result_id.to_string()))
        .returning(|_| Ok(None));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/results/{}", result_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_get_job_results_success() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_repo = MockScrapingRepository::new();
    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_task_results()
        .with(eq(task_id.to_string()))
        .returning(|_| Ok(vec![]));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/results/task/{}", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn test_get_scraper_results_success() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_repo = MockScrapingRepository::new();

    // Setup expectations
    mock_repo
        .expect_get_results_by_store()
        .with(eq("Bricodepot"))
        .returning(|_| Ok(vec![]));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri("/results/store/Bricodepot")
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn test_export_results_success() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_repo = MockScrapingRepository::new();
    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_task_results()
        .with(eq(task_id.to_string()))
        .returning(|_| Ok(vec![]));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/results/task/{}/export", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    assert_eq!(resp.headers().get("content-type").unwrap(), "text/csv");
} 