// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

/// Token operation errors.
#[derive(Debug, Error)]
pub enum TokenError {
    /// Redis data store access error.
    #[error("data store access error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("error while JSON processing: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic token error.
    #[error("token error: {0}")]
    Custom(String),

    #[error("token is invalid or expired")]
    InvalidToken,
}
