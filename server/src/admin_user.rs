// SPDX-License-Identifier: MIT

use std::future::Future;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest, error};
use tracing::warn;

use crate::app_data::AppData;
use crate::token::TokenError;

/// Represents an admin user for administrative operations
#[derive(Clone, Debug)]
pub struct AdminUser;

impl FromRequest for AdminUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let app_data = get_app_data(&req)?;
            let token = extract_admin_token_from_header(&req)?;

            match app_data.token_validator.validate_admin_token(&token).await {
                Ok(()) => Ok(AdminUser),
                Err(TokenError::InvalidToken) => Err(error::ErrorForbidden("Invalid admin token")),
                Err(e) => {
                    warn!("Admin token validation failed: {}", e);
                    Err(error::ErrorInternalServerError(
                        "Admin token validation failed",
                    ))
                }
            }
        })
    }
}

/// Extract the application data from the request
fn get_app_data(req: &HttpRequest) -> Result<actix_web::web::Data<AppData>, Error> {
    req.app_data::<actix_web::web::Data<AppData>>()
        .ok_or_else(|| error::ErrorInternalServerError("App data not found"))
        .cloned()
}

/// Extract and clean the admin token from the Authorization header
fn extract_admin_token_from_header(req: &HttpRequest) -> Result<String, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| error::ErrorUnauthorized("Authorization header required"))?;

    auth_header
        .strip_prefix("Bearer ")
        .map(|token| token.trim().to_string())
        .ok_or_else(|| error::ErrorUnauthorized("Invalid authorization format"))
}
