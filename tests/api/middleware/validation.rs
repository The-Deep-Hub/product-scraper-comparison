use actix_web::{test, web, App, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::api::middleware::validation::{validate_email, validate_password, validate_passwords_match, validate_request};

#[derive(Debug, Serialize, Deserialize)]
struct TestRequest {
    email: String,
    password: String,
    confirm_password: Option<String>,
}

async fn test_handler(data: web::Json<TestRequest>) -> HttpResponse {
    HttpResponse::Ok().json(data.0)
}

async fn setup_test_app() -> impl actix_web::dev::Service<actix_web::dev::ServiceRequest> {
    test::init_service(
        App::new()
            .route("/test", web::post().to(test_handler))
    ).await
}

#[test]
fn test_email_validation() {
    let valid_emails = vec![
        "test@example.com",
        "user.name@domain.co.uk",
        "user+tag@example.com",
        "first.last@subdomain.example.com",
    ];

    let invalid_emails = vec![
        "invalid-email",
        "@domain.com",
        "user@",
        "user@.com",
        "user@domain.",
        "user name@domain.com",
    ];

    for email in valid_emails {
        assert!(validate_email(email));
    }

    for email in invalid_emails {
        assert!(!validate_email(email));
    }
}

#[test]
fn test_password_validation() {
    let valid_passwords = vec![
        "Password123!",
        "StrongP@ss1",
        "C0mpl3x!Pass",
        "Sup3r$3cur3",
    ];

    let invalid_passwords = vec![
        "weak",
        "nodigits!",
        "12345678",
        "UPPERCASE123",
        "lowercase123",
        "Short1!",
    ];

    for password in valid_passwords {
        assert!(validate_password(password));
    }

    for password in invalid_passwords {
        assert!(!validate_password(password));
    }
}

#[test]
fn test_passwords_match() {
    assert!(validate_passwords_match("Password123!", "Password123!"));
    assert!(!validate_passwords_match("Password123!", "DifferentPass123!"));
}

#[actix_rt::test]
async fn test_validation_middleware_valid_request() {
    let app = setup_test_app().await;

    let request_data = TestRequest {
        email: "test@example.com".to_string(),
        password: "Password123!".to_string(),
        confirm_password: Some("Password123!".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&request_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_validation_middleware_invalid_email() {
    let app = setup_test_app().await;

    let request_data = TestRequest {
        email: "invalid-email".to_string(),
        password: "Password123!".to_string(),
        confirm_password: Some("Password123!".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&request_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_validation_middleware_invalid_password() {
    let app = setup_test_app().await;

    let request_data = TestRequest {
        email: "test@example.com".to_string(),
        password: "weak".to_string(),
        confirm_password: Some("weak".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&request_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_validation_middleware_password_mismatch() {
    let app = setup_test_app().await;

    let request_data = TestRequest {
        email: "test@example.com".to_string(),
        password: "Password123!".to_string(),
        confirm_password: Some("DifferentPass123!".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&request_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_validation_middleware_missing_fields() {
    let app = setup_test_app().await;

    // Missing email
    let request_data = serde_json::json!({
        "password": "Password123!",
        "confirm_password": "Password123!"
    });

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&request_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);

    // Missing password
    let request_data = serde_json::json!({
        "email": "test@example.com",
        "confirm_password": "Password123!"
    });

    let req = test::TestRequest::post()
        .uri("/test")
        .set_json(&request_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
} 