// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest, error};
use futures_util::StreamExt;
use serde::de::DeserializeOwned;

use super::size_limit;
use super::user::User;

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
        let mut payload = payload.take();

        Box::pin(async move {
            let user = User::extract(&req).await?;
            let size_limit = user.upload_size_limit.map(size_limit::calculate);

            // Stream the payload and enforce size limit during upload
            let mut body = actix_web::web::BytesMut::new();
            let mut total_size = 0usize;

            while let Some(chunk) = payload.next().await {
                let chunk = chunk.map_err(|e| {
                    error::ErrorBadRequest(format!("Failed to read request body: {e}"))
                })?;

                total_size += chunk.len();

                if let Some(limit) = size_limit
                    && total_size > limit
                {
                    return Err(error::ErrorPayloadTooLarge(format!(
                        "Upload size limit exceeded. Maximum allowed: {limit} bytes"
                    )));
                }

                body.extend_from_slice(&chunk);
            }

            let json = serde_json::from_slice::<T>(&body)
                .map_err(|e| error::ErrorBadRequest(format!("Invalid JSON: {e}")))?;

            Ok(SizeLimitedJson(json))
        })
    }
}
