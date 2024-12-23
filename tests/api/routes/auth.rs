use actix_web::{test, web, App};
use rust_scraper::api::{
    middleware::auth::{AuthMiddleware, AuthConfig, Role},
    models::auth::{RegisterRequest, LoginRequest},
    routes::auth::auth_routes,
};

#[actix_rt::test]
async fn test_register_endpoint() {
    // Setup
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let auth_middleware = AuthMiddleware::new(config);

    let register_req = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        name: "Test User".to_string(),
        role: Some(Role::Basic),
    };

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(auth_middleware.clone()))
            .service(auth_routes())
    ).await;

    // Create admin token
    let admin_token = auth_middleware.generate_token("admin@example.com", Role::Admin).unwrap();

    // Test registration with admin token
    let resp = test::TestRequest::post()
        .uri("/auth/register")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&register_req)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);
}

#[actix_rt::test]
async fn test_register_endpoint_unauthorized() {
    // Setup
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let auth_middleware = AuthMiddleware::new(config);

    let register_req = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        name: "Test User".to_string(),
        role: Some(Role::Basic),
    };

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(auth_middleware.clone()))
            .service(auth_routes())
    ).await;

    // Create basic user token
    let basic_token = auth_middleware.generate_token("user@example.com", Role::Basic).unwrap();

    // Test registration with basic token (should fail)
    let resp = test::TestRequest::post()
        .uri("/auth/register")
        .insert_header(("Authorization", format!("Bearer {}", basic_token)))
        .set_json(&register_req)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 401);
}

#[actix_rt::test]
async fn test_login_endpoint() {
    // Setup
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let auth_middleware = AuthMiddleware::new(config);

    let login_req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(auth_middleware))
            .service(auth_routes())
    ).await;

    // Test login
    let resp = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_req)
        .send_request(&app)
        .await;

    assert_eq!(resp.status(), 200);
} 