use actix_web::{test, web, App, HttpResponse};
use redis::Commands;
use std::time::Duration;
use tokio::time::sleep;
use crate::api::middleware::rate_limit::{RateLimiter, RateLimitConfig};

async fn test_handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn setup_test_app(
    max_requests: u32,
    window_secs: u32,
) -> (
    impl actix_web::dev::Service<actix_web::dev::ServiceRequest>,
    redis::Client,
) {
    let redis_client = redis::Client::open("redis://localhost").expect("Failed to create Redis client");
    
    let config = RateLimitConfig {
        max_requests,
        window_secs,
    };
    
    let rate_limiter = RateLimiter::new(redis_client.clone(), config);
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(rate_limiter))
            .route("/test", web::get().to(test_handler))
    ).await;
    
    (app, redis_client)
}

#[actix_rt::test]
async fn test_rate_limit_basic() {
    let (app, redis) = setup_test_app(2, 5).await;
    let test_ip = "192.168.1.1";
    
    // Clear any existing rate limit data
    let mut redis_conn = redis.get_connection().unwrap();
    let _: () = redis_conn.del(format!("rate_limit:{}", test_ip)).unwrap();
    
    // First request should succeed
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", test_ip))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // Second request should succeed
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", test_ip))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // Third request should fail (rate limited)
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", test_ip))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::TOO_MANY_REQUESTS);
}

#[actix_rt::test]
async fn test_rate_limit_window_reset() {
    let window_secs = 2;
    let (app, redis) = setup_test_app(1, window_secs).await;
    let test_ip = "192.168.1.2";
    
    // Clear any existing rate limit data
    let mut redis_conn = redis.get_connection().unwrap();
    let _: () = redis_conn.del(format!("rate_limit:{}", test_ip)).unwrap();
    
    // First request should succeed
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", test_ip))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // Second request should fail (rate limited)
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", test_ip))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::TOO_MANY_REQUESTS);
    
    // Wait for window to reset
    sleep(Duration::from_secs((window_secs + 1) as u64)).await;
    
    // Request after window reset should succeed
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", test_ip))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_rate_limit_multiple_ips() {
    let (app, redis) = setup_test_app(1, 5).await;
    let ip1 = "192.168.1.3";
    let ip2 = "192.168.1.4";
    
    // Clear any existing rate limit data
    let mut redis_conn = redis.get_connection().unwrap();
    let _: () = redis_conn.del(format!("rate_limit:{}", ip1)).unwrap();
    let _: () = redis_conn.del(format!("rate_limit:{}", ip2)).unwrap();
    
    // First IP's request should succeed
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", ip1))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // Second IP's request should succeed (different IP)
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", ip2))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // First IP's second request should fail
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", ip1))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::TOO_MANY_REQUESTS);
    
    // Second IP's second request should fail
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-Real-IP", ip2))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::TOO_MANY_REQUESTS);
}

#[actix_rt::test]
async fn test_rate_limit_missing_ip() {
    let (app, _) = setup_test_app(1, 5).await;
    
    // Request without IP header should fail gracefully
    let req = test::TestRequest::get()
        .uri("/test")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
} 