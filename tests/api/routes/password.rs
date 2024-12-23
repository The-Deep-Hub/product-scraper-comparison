use actix_web::{test, web, App};
use mongodb::Database;
use redis::aio::ConnectionManager;
use serde_json::json;

use crate::{
    api::{
        middleware::validation::{PasswordReset, PasswordUpdate},
        routes::password::{request_reset, confirm_reset},
    },
    db::repositories::user::UserRepository,
};

async fn setup_test_app(
    db: Database,
    redis: ConnectionManager,
) -> impl actix_web::dev::Service<actix_web::dev::Request> {
    App::new()
        .app_data(web::Data::new(UserRepository::new(&db)))
        .app_data(web::Data::new(redis))
        .service(
            web::scope("/password")
                .route("/reset", web::post().to(request_reset))
                .route("/reset/confirm", web::post().to(confirm_reset)),
        )
}

#[actix_web::test]
async fn test_request_reset_success() {
    // Setup test database and Redis
    let db = mongodb::Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap()
        .database("test_db");
    let redis = redis::Client::open("redis://localhost")
        .unwrap()
        .get_async_connection()
        .await
        .unwrap();

    let app = test::init_service(setup_test_app(db.clone(), redis)).await;

    // Create test user first
    let user_repo = UserRepository::new(&db);
    let user = crate::db::models::user::User {
        id: None,
        email: "test@example.com".to_string(),
        password_hash: bcrypt::hash("Test123!@#", bcrypt::DEFAULT_COST).unwrap(),
        role: crate::api::middleware::auth::Role::Basic,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    user_repo.create(user).await.unwrap();

    let reset_data = PasswordReset {
        email: "test@example.com".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/password/reset")
        .set_json(&reset_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("token").is_some());

    // Cleanup
    db.collection::<mongodb::bson::Document>("users")
        .drop(None)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_request_reset_nonexistent_email() {
    // Setup test database and Redis
    let db = mongodb::Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap()
        .database("test_db");
    let redis = redis::Client::open("redis://localhost")
        .unwrap()
        .get_async_connection()
        .await
        .unwrap();

    let app = test::init_service(setup_test_app(db.clone(), redis)).await;

    let reset_data = PasswordReset {
        email: "nonexistent@example.com".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/password/reset")
        .set_json(&reset_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // Cleanup
    db.collection::<mongodb::bson::Document>("users")
        .drop(None)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_confirm_reset_success() {
    // Setup test database and Redis
    let db = mongodb::Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap()
        .database("test_db");
    let redis = redis::Client::open("redis://localhost")
        .unwrap()
        .get_async_connection()
        .await
        .unwrap();

    let app = test::init_service(setup_test_app(db.clone(), redis.clone())).await;

    // Create test user first
    let user_repo = UserRepository::new(&db);
    let user = crate::db::models::user::User {
        id: None,
        email: "test@example.com".to_string(),
        password_hash: bcrypt::hash("Test123!@#", bcrypt::DEFAULT_COST).unwrap(),
        role: crate::api::middleware::auth::Role::Basic,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let created_user = user_repo.create(user).await.unwrap();

    // Request password reset
    let reset_data = PasswordReset {
        email: "test@example.com".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/password/reset")
        .set_json(&reset_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let reset_token = body["token"].as_str().unwrap();

    // Confirm password reset
    let confirm_data = PasswordUpdate {
        password: "NewTest123!@#".to_string(),
        confirm_password: "NewTest123!@#".to_string(),
        reset_token: reset_token.to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/password/reset/confirm")
        .set_json(&confirm_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify password was updated
    let user = user_repo
        .find_by_id(created_user.id.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert!(bcrypt::verify("NewTest123!@#", &user.password_hash).unwrap());

    // Cleanup
    db.collection::<mongodb::bson::Document>("users")
        .drop(None)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_confirm_reset_invalid_token() {
    // Setup test database and Redis
    let db = mongodb::Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap()
        .database("test_db");
    let redis = redis::Client::open("redis://localhost")
        .unwrap()
        .get_async_connection()
        .await
        .unwrap();

    let app = test::init_service(setup_test_app(db.clone(), redis)).await;

    let confirm_data = PasswordUpdate {
        password: "NewTest123!@#".to_string(),
        confirm_password: "NewTest123!@#".to_string(),
        reset_token: "invalid_token".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/password/reset/confirm")
        .set_json(&confirm_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // Cleanup
    db.collection::<mongodb::bson::Document>("users")
        .drop(None)
        .await
        .unwrap();
} 