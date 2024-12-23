use actix_web::{test, web::Data, App};
use rust_scraper::api::{
    models::auth::{LoginRequest, RegisterRequest},
    routes::auth_config,
};
use rust_scraper::{
    db::repositories::user::UserRepository,
    services::cache::RedisCacheService,
};
use mongodb::Client;
use redis::aio::ConnectionManager;

async fn setup_test_services() -> (Data<UserRepository>, Data<RedisCacheService>) {
    // Set up MongoDB
    let mongo_uri = std::env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string());
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .unwrap();
    let db = client.database("test_db");
    // Clear users collection before each test
    db.collection::<mongodb::bson::Document>("users")
        .drop(None)
        .await
        .ok();
    let user_repo = Data::new(UserRepository::new(db));

    // Set up Redis
    let redis_uri = std::env::var("REDIS_URI")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = redis::Client::open(redis_uri).unwrap();
    let redis_manager = ConnectionManager::new(redis_client).await.unwrap();
    let cache_service = Data::new(RedisCacheService::new(redis_manager));

    (user_repo, cache_service)
}

#[actix_rt::test]
async fn test_register_route() {
    let (repo, cache) = setup_test_services().await;

    let app = test::init_service(
        App::new()
            .app_data(repo)
            .app_data(cache)
            .configure(auth_config),
    )
    .await;

    let request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        password_confirmation: "password123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_login_route() {
    let (repo, cache) = setup_test_services().await;

    let app = test::init_service(
        App::new()
            .app_data(repo.clone())
            .app_data(cache.clone())
            .configure(auth_config),
    )
    .await;

    // First register a user
    let register_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        password_confirmation: "password123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Registration should succeed");

    // Then try to login
    let login_request = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "Login should succeed with valid credentials");

    // Verify response content
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("token").is_some(), "Response should contain a token");
    assert_eq!(body["email"], "test@example.com", "Response should contain the correct email");
    assert_eq!(body["role"], "user", "Response should contain the user role");
    assert_eq!(body["message"], "Login successful", "Response should indicate success");

    // Test login with wrong password
    let wrong_login_request = LoginRequest {
        email: "test@example.com".to_string(),
        password: "wrongpassword".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&wrong_login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401, "Login should fail with wrong password");
} 