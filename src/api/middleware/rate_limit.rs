use std::future::{ready, Ready};
use std::time::Duration;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    web::Data,
    Error,
};
use futures_util::future::LocalBoxFuture;
use log::error;
use redis::aio::ConnectionManager;

pub struct RateLimiter {
    requests: u32,
    duration: Duration,
    redis: Data<ConnectionManager>,
}

impl RateLimiter {
    pub fn new(requests: u32, duration: Duration, redis: Data<ConnectionManager>) -> Self {
        Self {
            requests,
            duration,
            redis,
        }
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
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service,
            requests: self.requests,
            duration: self.duration,
            redis: self.redis.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    requests: u32,
    duration: Duration,
    redis: Data<ConnectionManager>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let key = format!("rate_limit:{}:{}", ip, req.path());
        let requests = self.requests;
        let duration = self.duration;
        let redis = self.redis.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut conn = redis.get_ref().clone();

            let count: Option<u32> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| {
                    error!("Redis get error: {}", e);
                    Error::from(ErrorTooManyRequests("Rate limit error"))
                })?;

            match count {
                Some(count) if count >= requests => {
                    Err(Error::from(ErrorTooManyRequests("Too many requests")))
                }
                Some(_) => {
                    let _: () = redis::cmd("INCR")
                        .arg(&key)
                        .query_async(&mut conn)
                        .await
                        .map_err(|e| {
                            error!("Redis incr error: {}", e);
                            Error::from(ErrorTooManyRequests("Rate limit error"))
                        })?;
                    fut.await
                }
                None => {
                    let _: () = redis::cmd("SETEX")
                        .arg(&key)
                        .arg(duration.as_secs() as u64)
                        .arg(1)
                        .query_async(&mut conn)
                        .await
                        .map_err(|e| {
                            error!("Redis set error: {}", e);
                            Error::from(ErrorTooManyRequests("Rate limit error"))
                        })?;
                    fut.await
                }
            }
        })
    }
} 