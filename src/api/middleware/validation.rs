use actix_web::{
    body::{EitherBody, MessageBody, BoxBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpResponse,
};
use serde::de::DeserializeOwned;
use std::{
    future::{ready, Ready, Future},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};
use validator::Validate;
use serde::Deserialize;

pub struct ValidateRequest<T>(std::marker::PhantomData<T>);

impl<T> ValidateRequest<T> {
    pub fn new() -> Self {
        ValidateRequest(std::marker::PhantomData)
    }
}

impl<S, B, T> Transform<S, ServiceRequest> for ValidateRequest<T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    T: DeserializeOwned + Validate + FromRequest + 'static,
    T::Error: Into<Error>,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ValidateRequestMiddleware<S, T>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ValidateRequestMiddleware {
            service: Rc::new(service),
            _phantom: std::marker::PhantomData,
        }))
    }
}

pub struct ValidateRequestMiddleware<S, T> {
    service: Rc<S>,
    _phantom: std::marker::PhantomData<T>,
}

impl<S, B, T> Service<ServiceRequest> for ValidateRequestMiddleware<S, T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
    T: DeserializeOwned + Validate + FromRequest + 'static,
    T::Error: Into<Error>,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let http_req = req.parts().0.clone();
            if let Ok(payload) = T::extract(&http_req).await {
                if let Err(errors) = payload.validate() {
                    let res = HttpResponse::BadRequest().json(errors);
                    return Ok(ServiceResponse::new(http_req, res).map_into_left_body());
                }
            }
            let res = svc.call(req).await?;
            Ok(res.map_into_right_body())
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordReset {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordUpdate {
    #[validate(length(min = 8, max = 100))]
    pub password: String,
    pub reset_token: String,
} 