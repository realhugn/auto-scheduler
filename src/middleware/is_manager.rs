use actix_web::{error::ErrorUnauthorized, Error, HttpMessage};
use std::future::{ready, Ready};

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;
use crate::utils::TokenClaims;

pub struct IsManager;
impl<S, B> Transform<S, ServiceRequest> for IsManager
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = IsManagerHiMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IsManagerHiMiddleware {
            service
        }))
    }
}

pub struct IsManagerHiMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for IsManagerHiMiddleware<S>
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
        let claims = req.extensions().get::<TokenClaims>().cloned();
        if claims.is_none() {
            return Box::pin(async { Err(ErrorUnauthorized("err1")) });
        }
        let claims = claims.unwrap();
        if claims.role != "Manager" {
            return Box::pin(async { Err(ErrorUnauthorized("err2")) });
        }
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}