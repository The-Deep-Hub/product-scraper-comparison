use actix_web::{test, web, App};
use mockall::predicate::*;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api::routes::job::config,
    db::repositories::scraping::MockScrapingRepository,
};

#[actix_web::test]
async fn test_get_tasks_success() {
    // Create mock
    let mut mock_repo = MockScrapingRepository::new();

    // Setup expectations
    mock_repo
        .expect_get_tasks()
        .returning(|_, _, _| Ok(vec![]));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri("/tasks")
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn test_get_task_success() {
    // Create mock
    let mut mock_repo = MockScrapingRepository::new();
    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_task()
        .with(eq(task_id.to_string()))
        .returning(|_| Ok(Some(/* create mock task */)));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/tasks/{}", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_get_task_not_found() {
    // Create mock
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
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/tasks/{}", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_retry_task_success() {
    // Create mock
    let mut mock_repo = MockScrapingRepository::new();
    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_task()
        .with(eq(task_id.to_string()))
        .returning(|_| Ok(Some(/* create mock task */)));

    mock_repo
        .expect_update_task_status()
        .returning(|_, _, _| Ok(true));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::post()
        .uri(&format!("/tasks/{}/retry", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "retrying");
}

#[actix_web::test]
async fn test_get_task_metrics_success() {
    // Create mock
    let mut mock_repo = MockScrapingRepository::new();
    let task_id = Uuid::new_v4();

    // Setup expectations
    mock_repo
        .expect_get_task_metrics()
        .with(eq(task_id.to_string()))
        .returning(|_| Ok(Some(/* create mock metrics */)));

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(mock_repo))
            .configure(config),
    )
    .await;

    // Create test request
    let req = test::TestRequest::get()
        .uri(&format!("/tasks/{}/metrics", task_id))
        .to_request();

    // Send request and check response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
} 