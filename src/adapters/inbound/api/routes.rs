use actix_web::{get, post, web, HttpResponse, Responder};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, error};

use super::models::{SearchRequest, SearchResponse, TaskStatusResponse};
use crate::domain::ports::inbound::ProductSearchPort;

const TASK_TIMEOUT: Duration = Duration::from_secs(180); // 3 minutes
const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub struct ApiRoutes<T: ProductSearchPort + 'static> {
    search_service: T,
}

impl<T: ProductSearchPort + 'static> ApiRoutes<T> {
    pub fn new(search_service: T) -> Self {
        Self { search_service }
    }

    pub fn configure(search_service: T) -> impl FnOnce(&mut web::ServiceConfig) {
        move |cfg: &mut web::ServiceConfig| {
            let routes = ApiRoutes::new(search_service);
            cfg.app_data(web::Data::new(routes))
                .service(
                    web::resource("/search")
                        .route(web::post().to(Self::search_products))
                )
                .service(
                    web::resource("/product")
                        .route(web::get().to(Self::get_product_details))
                );
        }
    }

    async fn search_products(
        request: web::Json<SearchRequest>,
        routes: web::Data<ApiRoutes<T>>,
    ) -> impl Responder {
        info!("Received search request: {:?}", request);
        
        // Validate request
        if request.query.trim().is_empty() {
            error!("Empty search query received");
            return HttpResponse::BadRequest().json(TaskStatusResponse {
                status: "error".to_string(),
                stores: None,
                error: Some("Search query cannot be empty".to_string()),
            });
        }
        
        // Start the search process
        match timeout(TASK_TIMEOUT, async {
            match request.store {
                Some(store) => {
                    routes.search_service.search_store_products(store, &request.query).await
                },
                None => {
                    routes.search_service.search_products(&request.query).await
                }
            }
        }).await {
            Ok(result) => match result {
                Ok(products) => {
                    // Group products by store
                    let mut stores = std::collections::HashMap::new();
                    for product in products {
                        stores
                            .entry(product.store().to_string().to_lowercase())
                            .or_insert_with(Vec::new)
                            .push(product);
                    }
                    
                    HttpResponse::Ok().json(TaskStatusResponse {
                        status: "completed".to_string(),
                        stores: Some(stores),
                        error: None,
                    })
                },
                Err(e) => {
                    error!("Search failed: {}", e);
                    HttpResponse::InternalServerError().json(TaskStatusResponse {
                        status: "error".to_string(),
                        stores: None,
                        error: Some(e.to_string()),
                    })
                }
            },
            Err(_) => {
                info!("Search timeout for query: {}", request.query);
                HttpResponse::Accepted().json(SearchResponse {
                    task_id: "".to_string(), // We don't have task IDs in the new architecture
                    status: "processing".to_string(),
                    message: "Search is still processing. Please try again later.".to_string(),
                })
            }
        }
    }

    async fn get_product_details(
        url: web::Query<String>,
        routes: web::Data<ApiRoutes<T>>,
    ) -> impl Responder {
        match routes.search_service.get_product_details(&url).await {
            Ok(product) => HttpResponse::Ok().json(product),
            Err(e) => {
                error!("Failed to get product details: {}", e);
                HttpResponse::InternalServerError().json(TaskStatusResponse {
                    status: "error".to_string(),
                    stores: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }
} 