// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;

use crate::token::{TokenData, TokenError};

#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Validate token and return metadata.
    async fn validate_user_token(&self, token: &str) -> Result<TokenData, TokenError>;

    /// Validate admin token.
    async fn validate_admin_token(&self, token: &str) -> Result<(), TokenError>;
}
