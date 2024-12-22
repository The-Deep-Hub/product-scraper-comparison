#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use serde_json::Value;

    use super::*;

    #[actix_rt::test]
    async fn test_health_check() {
        // Arrange
        let app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::Logger::default())
                .configure(health_config),
        )
        .await;

        // Act
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert!(resp.status().is_success());

        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert_eq!(body["data"]["status"], "ok");
        assert!(body["data"]["version"].as_str().is_some());
    }
} 