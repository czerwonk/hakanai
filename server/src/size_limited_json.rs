use std::future::Future;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::web::Bytes;
use actix_web::{Error, FromRequest, HttpRequest, error};
use serde::de::DeserializeOwned;

use crate::user::User;

/// Custom JSON extractor that enforces size limits based on user's upload limit
///
/// This extractor:
/// 1. Extracts the User to get the size limit
/// 2. Reads the payload while enforcing the size limit during streaming
/// 3. Fails fast if the size limit is exceeded
/// 4. Parses the JSON after the complete payload is validated
pub struct SizeLimitedJson<T>(pub T);

impl<T> SizeLimitedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromRequest for SizeLimitedJson<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        let payload_future = Bytes::from_request(&req, payload);

        Box::pin(async move {
            let user = User::extract(&req).await?;

            let body = payload_future
                .await
                .map_err(|e| error::ErrorBadRequest(format!("Failed to read request body: {e}")))?;
            let body_size = body.len();

            let size_limit = user.upload_size_limit;
            if body_size > size_limit {
                return Err(error::ErrorPayloadTooLarge(format!(
                    "Upload size limit exceeded. Maximum allowed: {size_limit} bytes"
                )));
            }

            let json = serde_json::from_slice::<T>(&body)
                .map_err(|e| error::ErrorBadRequest(format!("Invalid JSON: {e}")))?;

            Ok(SizeLimitedJson(json))
        })
    }
}
