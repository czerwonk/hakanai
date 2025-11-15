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
