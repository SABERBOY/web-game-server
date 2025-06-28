use crate::models::Claims;
use crate::utils::JwtUtil;
use actix_web::{
    dev::ServiceRequest, error::ErrorUnauthorized, Error, FromRequest, HttpMessage, HttpRequest,
};
use actix_web_httpauth::extractors::{
    bearer::{self, BearerAuth},
    AuthenticationError,
};
use std::future::{ready, Ready};

pub struct AuthUser {
    pub user_id: String,
    pub username: String,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let result = req
            .extensions()
            .get::<Claims>()
            .map(|claims| AuthUser {
                user_id: claims.sub.clone(),
                username: claims.username.clone(),
            })
            .ok_or_else(|| ErrorUnauthorized("Unauthorized"));

        ready(result)
    }
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    
    match JwtUtil::verify_token(token) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            let config = req
                .app_data::<bearer::Config>()
                .cloned()
                .unwrap_or_default();
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}