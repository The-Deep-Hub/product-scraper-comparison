use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use validator::Validate;

pub struct ValidationMiddleware<T>(std::marker::PhantomData<T>);

impl<T> ValidationMiddleware<T> {
    pub fn new() -> Self {
        ValidationMiddleware(std::marker::PhantomData)
    }
}

impl<S, B, T> Transform<S, ServiceRequest> for ValidationMiddleware<T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    T: Validate + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ValidationMiddlewareService<S, T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ValidationMiddlewareService {
            service,
            _phantom: std::marker::PhantomData,
        })
    }
}

pub struct ValidationMiddlewareService<S, T> {
    service: S,
    _phantom: std::marker::PhantomData<T>,
}

impl<S, B, T> Service<ServiceRequest> for ValidationMiddlewareService<S, T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    T: Validate + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        Box::pin(self.service.call(req))
    }
}

pub async fn validate_request<T>(_req: &ServiceRequest, data: &T) -> Result<(), Error>
where
    T: Validate,
{
    data.validate()
        .map_err(|e| Error::from(actix_web::error::ErrorBadRequest(e.to_string())))
} 