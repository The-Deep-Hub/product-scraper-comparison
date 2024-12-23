use actix_web::{
    test,
    web::self,
};
use rust_scraper::api::{
    middleware::auth::AuthMiddleware,
    routes::health_config,
};

#[actix_rt::test]
async fn test_auth_middleware() {
    let app = test::init_service(
        actix_web::App::new()
            .service(
                web::scope("/test")
                    .wrap(AuthMiddleware)
                    .configure(health_config)
            )
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 401);
} 