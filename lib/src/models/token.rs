// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Request model for creating user tokens via admin API
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenRequest {
    /// Optional upload size limit in bytes
    pub upload_size_limit: Option<i64>,
    /// TTL in seconds
    pub ttl_seconds: u64,
    /// Wether it is a one time use token
    #[serde(default)]
    pub one_time: bool,
}

impl CreateTokenRequest {
    /// Create a new CreateTokenRequest
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            upload_size_limit: None,
            ttl_seconds,
            one_time: false,
        }
    }

    /// Set the upload size limit
    #[cfg(any(test, feature = "testing"))]
    pub fn with_upload_size_limit(mut self, limit: i64) -> Self {
        self.upload_size_limit = Some(limit);
        self
    }

    /// Set the one time use flag
    #[cfg(any(test, feature = "testing"))]
    pub fn with_one_time(mut self) -> Self {
        self.one_time = true;
        self
    }
}

/// Response model for creating user tokens via admin API
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenResponse {
    /// The generated token
    pub token: String,
}

impl Zeroize for CreateTokenResponse {
    fn zeroize(&mut self) {
        self.token.zeroize();
    }
}

impl Drop for CreateTokenResponse {
    fn drop(&mut self) {
        self.zeroize();
    }
}
