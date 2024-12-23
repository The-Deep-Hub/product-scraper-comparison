use actix_web::{test, web, App};
use mockall::predicate::*;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::routes::scraper::{config, SearchRequest, Store},
    services::{
        cache::MockCacheService,
        queue::MockQueueService,
    },
    db::repositories::scraping::MockScrapingRepository,
};

#[actix_web::test]
async fn test_search_products_success() {
    // Create mocks
    let mut mock_cache = MockCacheService::new();
    let mut mock_queue = MockQueueService::new();
    let mut mock_repo = MockScrapingRepository::new();

    // Setup expectations
    mock_cache
        .expect_get_search_results()
        .with(eq("hammer"), eq(vec![Store::Bricodepot]))
        .returning(|_, _| Ok(None));

    let task_id = Uuid::new_v4();
    mock_queue
        .expect_create_scraping_tasks()
        .returning(move |_, _| Ok(vec![]));

    mock_repo
        .expect_create_task()
        .returning(|_| Ok("task_id".to_string()));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_queue))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::post()
        .uri("/search")
        .set_json(json!({
            "query": "hammer",
            "stores": ["Bricodepot"],
            "num_products": 10
        }))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["query"], "hammer");
    assert_eq!(body["status"], "pending");
}

#[actix_web::test]
async fn test_search_products_validation_failure() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mock_queue = MockQueueService::new();
    let mock_repo = MockScrapingRepository::new();

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_queue))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Test with empty query
    let req = test::TestRequest::post()
        .uri("/search")
        .set_json(json!({
            "query": "",
            "stores": ["Bricodepot"],
            "num_products": 10
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // Test with invalid num_products
    let req = test::TestRequest::post()
        .uri("/search")
        .set_json(json!({
            "query": "hammer",
            "stores": ["Bricodepot"],
            "num_products": 0
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_get_search_results_success() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_queue = MockQueueService::new();
    let mut mock_repo = MockScrapingRepository::new();

    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_task()
        .with(eq(task_id.to_string()))
        .returning(|_| Ok(Some(/* create mock task */)));

    mock_repo
        .expect_get_results_by_query()
        .returning(|_, _, _, _| Ok(vec![]));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_queue))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/search/{}", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_search_results_not_found() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_queue = MockQueueService::new();
    let mut mock_repo = MockScrapingRepository::new();

    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_task()
        .with(eq(task_id.to_string()))
        .returning(|_| Ok(None));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_queue))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/search/{}", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_cancel_search_success() {
    // Create mocks
    let mock_cache = MockCacheService::new();
    let mut mock_queue = MockQueueService::new();
    let mut mock_repo = MockScrapingRepository::new();

    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_queue
        .expect_cancel_task()
        .returning(|_| Ok(()));

    mock_repo
        .expect_update_task_status()
        .returning(|_, _, _| Ok(true));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_cache))
            .app_data(web::Data::new(mock_queue))
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::post()
        .uri(&format!("/search/{}/cancel", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "cancelled");
} 