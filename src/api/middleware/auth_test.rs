#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    fn create_test_middleware() -> AuthMiddleware {
        AuthMiddleware::new(AuthConfig {
            jwt_secret: "test-secret".to_string(),
            token_expiration: 3600,
        })
    }

    #[test]
    fn test_token_generation_and_verification() {
        let middleware = create_test_middleware();
        
        // Generate token
        let token = middleware.generate_token("user123", Role::Basic).unwrap();
        
        // Verify token
        let claims = middleware.verify_token(&token).unwrap();
        
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, Role::Basic);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_invalid_token() {
        let middleware = create_test_middleware();
        let result = middleware.verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[actix_rt::test]
    async fn test_auth_middleware() {
        let middleware = create_test_middleware();
        let token = middleware.generate_token("user123", Role::Basic).unwrap();
        
        // Create test request
        let req = test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_srv_request();
        
        // Test token extraction
        let extracted_token = middleware.extract_token(&req).unwrap();
        assert_eq!(extracted_token, token);
        
        // Test request verification
        let claims = middleware.verify_request(&req).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, Role::Basic);
    }

    #[actix_rt::test]
    async fn test_missing_auth_header() {
        let middleware = create_test_middleware();
        let req = test::TestRequest::default().to_srv_request();
        
        let result = middleware.extract_token(&req);
        assert!(result.is_err());
    }

    #[actix_rt::test]
    async fn test_invalid_auth_header() {
        let middleware = create_test_middleware();
        let req = test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, "NotBearer token"))
            .to_srv_request();
        
        let result = middleware.extract_token(&req);
        assert!(result.is_err());
    }

    #[actix_rt::test]
    async fn test_role_based_access() {
        let config = AuthConfig {
            jwt_secret: "test-secret".to_string(),
            token_expiration: 3600,
        };
        
        let middleware = AuthMiddleware::new(config.clone());
        
        // Generate admin token
        let admin_token = middleware.generate_token("admin123", Role::Admin).unwrap();
        let admin_req = test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
            .to_srv_request();
        
        // Generate basic user token
        let basic_token = middleware.generate_token("user123", Role::Basic).unwrap();
        let basic_req = test::TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", basic_token)))
            .to_srv_request();
        
        // Test admin access
        let admin_middleware = require_role(Role::Admin, config.clone());
        assert!(admin_middleware(admin_req.clone()).await.is_ok());
        assert!(admin_middleware(basic_req.clone()).await.is_err());
        
        // Test basic access
        let basic_middleware = require_role(Role::Basic, config);
        assert!(basic_middleware(basic_req).await.is_ok());
    }
} 