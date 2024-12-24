use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct RateLimiter {
    requests_per_second: u32,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        Self { requests_per_second }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterService {
            service: Rc::new(service),
            requests_per_second: self.requests_per_second,
            requests: Rc::new(Mutex::new(HashMap::new())),
        }))
    }
}

pub struct RateLimiterService<S> {
    service: Rc<S>,
    requests_per_second: u32,
    requests: Rc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let requests = self.requests.clone();
        let requests_per_second = self.requests_per_second;

        Box::pin(async move {
            let ip = req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("unknown")
                .to_string();

            let mut requests = requests.lock().await;
            let now = Instant::now();
            let window = Duration::from_secs(1);

            // Clean up old requests
            if let Some(times) = requests.get_mut(&ip) {
                times.retain(|&time| now.duration_since(time) < window);
            }

            // Check rate limit
            let count = requests.entry(ip.clone()).or_insert_with(Vec::new);
            if count.len() >= requests_per_second as usize {
                return Err(ErrorTooManyRequests("Too many requests"));
            }

            // Add new request
            count.push(now);
            drop(requests);

            service.call(req).await
        })
    }
} 