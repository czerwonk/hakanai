// SPDX-License-Identifier: MIT

use std::future::Future;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest, error};
use tracing::warn;

use crate::app_data::AppData;
use crate::token::TokenError;

/// User type for authentication and tracing
#[derive(Clone, Debug)]
pub enum UserType {
    Anonymous,
    Authenticated,
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserType::Anonymous => write!(f, "anonymous"),
            UserType::Authenticated => write!(f, "authenticated"),
        }
    }
}

/// Represents a user in the system, either authenticated or anonymous
#[derive(Clone, Debug)]
pub struct User {
    /// The effective upload size limit for this user in bytes
    pub upload_size_limit: usize,
    /// The type of user (anonymous or authenticated)
    pub user_type: UserType,
}

impl User {
    /// Create an authenticated user with a specific upload limit
    pub fn authenticated(upload_size_limit: usize) -> Self {
        Self {
            upload_size_limit,
            user_type: UserType::Authenticated,
        }
    }

    /// Create an anonymous user with a specific upload limit
    pub fn anonymous(upload_size_limit: usize) -> Self {
        Self {
            upload_size_limit,
            user_type: UserType::Anonymous,
        }
    }
}

impl FromRequest for User {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let app_data = get_app_data(&req)?;
            let token = extract_token_from_header(&req);

            match token {
                Some(token) => handle_authenticated_request(token, app_data).await,
                None => handle_anonymous_request(app_data),
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

/// Extract and clean the token from the Authorization header
fn extract_token_from_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|token| token.trim_start_matches("Bearer ").trim().to_string())
}

/// Handle a request with an authentication token
async fn handle_authenticated_request(
    token: String,
    app_data: actix_web::web::Data<AppData>,
) -> Result<User, Error> {
    match app_data.token_validator.validate_user_token(&token).await {
        Ok(token_data) => {
            let upload_size_limit = extract_upload_limit(token_data);
            Ok(User::authenticated(upload_size_limit))
        }
        Err(TokenError::InvalidToken) => Err(error::ErrorForbidden("Invalid token")),
        Err(e) => {
            warn!("Token validation failed: {}", e);
            Err(error::ErrorInternalServerError("Token validation failed"))
        }
    }
}

/// Extract the upload size limit from token data
fn extract_upload_limit(token_data: crate::token::TokenData) -> usize {
    token_data
        .upload_size_limit
        .map(|limit| limit as usize)
        .unwrap_or(usize::MAX) // No limit for authenticated users without specific limit
}

/// Handle a request without an authentication token
fn handle_anonymous_request(app_data: actix_web::web::Data<AppData>) -> Result<User, Error> {
    if app_data.anonymous_usage.allowed {
        Ok(User::anonymous(
            app_data.anonymous_usage.upload_size_limit as usize,
        ))
    } else {
        Err(error::ErrorUnauthorized("Authorization token required"))
    }
}
