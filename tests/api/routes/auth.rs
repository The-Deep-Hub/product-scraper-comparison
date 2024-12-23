use actix_web::{test, web::Data, App};
use rust_scraper::api::{
    models::auth::{LoginRequest, RegisterRequest},
    routes::auth_config,
};
use rust_scraper::db::repositories::user::UserRepository;
use mongodb::Client;

async fn setup_test_db() -> Data<UserRepository> {
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
    Data::new(UserRepository::new(db))
}

#[actix_rt::test]
async fn test_register_route() {
    let repo = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(repo)
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
    let repo = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(repo.clone())
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
    assert!(resp.status().is_success());

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
    assert!(resp.status().is_success());
} 