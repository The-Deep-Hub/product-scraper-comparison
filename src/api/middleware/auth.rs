use std::future::{ready, Ready};
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};

use crate::api::utils::validate_token;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract the token from the Authorization header
        let auth_header = req.headers().get("Authorization");
        let auth_header = match auth_header {
            Some(header) => header,
            None => {
                let (http_req, _) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "No authorization header"
                    }));
                return Box::pin(async move {
                    Ok(ServiceResponse::new(
                        http_req,
                        response.map_into_right_body(),
                    ))
                });
            }
        };

        // Parse the Bearer token
        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => {
                let (http_req, _) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Invalid authorization header format"
                    }));
                return Box::pin(async move {
                    Ok(ServiceResponse::new(
                        http_req,
                        response.map_into_right_body(),
                    ))
                });
            }
        };

        let token = match auth_str.strip_prefix("Bearer ") {
            Some(token) => token,
            None => {
                let (http_req, _) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Invalid token format"
                    }));
                return Box::pin(async move {
                    Ok(ServiceResponse::new(
                        http_req,
                        response.map_into_right_body(),
                    ))
                });
            }
        };

        // Validate the token
        match validate_token(token) {
            Ok(_claims) => {
                // Token is valid, proceed with the request
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res.map_into_left_body())
                })
            }
            Err(_) => {
                let (http_req, _) = req.into_parts();
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Invalid token"
                    }));
                Box::pin(async move {
                    Ok(ServiceResponse::new(
                        http_req,
                        response.map_into_right_body(),
                    ))
                })
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    User,
    Admin,
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::User => "user".to_string(),
            Role::Admin => "admin".to_string(),
        }
    }
} 