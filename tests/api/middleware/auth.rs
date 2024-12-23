use actix_web::{
    test::TestRequest,
    http::header,
};
use rust_scraper::api::middleware::auth::{AuthMiddleware, AuthConfig, Role};

#[test]
fn test_generate_token() {
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let middleware = AuthMiddleware::new(config);

    let token = middleware.generate_token("test@example.com", Role::Basic).unwrap();
    assert!(!token.is_empty());
}

#[test]
fn test_verify_token() {
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let middleware = AuthMiddleware::new(config);

    let token = middleware.generate_token("test@example.com", Role::Basic).unwrap();
    let claims = middleware.verify_token(&token).unwrap();

    assert_eq!(claims.sub, "test@example.com");
    assert_eq!(claims.role, Role::Basic);
}

#[test]
fn test_verify_request_valid_token() {
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let middleware = AuthMiddleware::new(config);

    let token = middleware.generate_token("test@example.com", Role::Basic).unwrap();
    let req = TestRequest::default()
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_srv_request();

    let claims = middleware.verify_request(&req).unwrap();
    assert_eq!(claims.sub, "test@example.com");
    assert_eq!(claims.role, Role::Basic);
}

#[test]
fn test_verify_request_missing_header() {
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let middleware = AuthMiddleware::new(config);

    let req = TestRequest::default().to_srv_request();
    let result = middleware.verify_request(&req);
    assert!(result.is_err());
}

#[test]
fn test_verify_request_invalid_header_format() {
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let middleware = AuthMiddleware::new(config);

    let req = TestRequest::default()
        .insert_header((header::AUTHORIZATION, "InvalidToken"))
        .to_srv_request();

    let result = middleware.verify_request(&req);
    assert!(result.is_err());
}

#[test]
fn test_verify_request_invalid_token() {
    let config = AuthConfig {
        jwt_secret: "test-secret".to_string(),
        token_expiration: 3600,
    };
    let middleware = AuthMiddleware::new(config);

    let req = TestRequest::default()
        .insert_header((header::AUTHORIZATION, "Bearer invalid.token.here"))
        .to_srv_request();

    let result = middleware.verify_request(&req);
    assert!(result.is_err());
} 