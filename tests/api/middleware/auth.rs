// use actix_web::{
//     test,
//     web::self,
//     http::header,
//     get, HttpResponse,
// };
// use rust_scraper::api::{
//     middleware::auth::{AuthMiddleware, Role},
//     routes::health_config,
//     utils::generate_token,
// };
// use std::env;

// // Protected test route
// #[get("/protected")]
// async fn protected_route() -> HttpResponse {
//     HttpResponse::Ok().json(serde_json::json!({
//         "status": "ok",
//         "message": "Protected route accessed successfully"
//     }))
// }

// #[actix_rt::test]
// async fn test_auth_middleware() {
//     // Set up JWT secret for testing
//     env::set_var("JWT_SECRET", "test-secret-key-for-testing-only");

//     let app = test::init_service(
//         actix_web::App::new()
//             .service(
//                 web::scope("/api")
//                     .wrap(AuthMiddleware)
//                     .service(protected_route)
//             )
//             .configure(health_config)
//     )
//     .await;

//     // Test public route (health check)
//     let req = test::TestRequest::get()
//         .uri("/health")
//         .to_request();

//     let resp = test::call_service(&app, req).await;
//     assert!(resp.status().is_success(), "Public route should be accessible without token");

//     // Test protected route without token
//     let req = test::TestRequest::get()
//         .uri("/api/protected")
//         .to_request();

//     let resp = test::call_service(&app, req).await;
//     assert_eq!(resp.status().as_u16(), 401, "Protected route should require token");

//     // Test protected route with valid token
//     let token = generate_token("test@example.com", &Role::User).expect("Failed to generate token");
//     let req = test::TestRequest::get()
//         .uri("/api/protected")
//         .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
//         .to_request();

//     let resp = test::call_service(&app, req).await;
//     assert!(resp.status().is_success(), "Protected route should be accessible with valid token");
// } 