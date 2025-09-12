// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use async_trait::async_trait;

use super::{TokenData, TokenError};

/// Abstraction for token storage operations.
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Gets token metadata by its hash.
    async fn get_token(&self, token_hash: &str) -> Result<Option<TokenData>, TokenError>;

    /// Store token with metadata.
    async fn store_token(
        &self,
        token_hash: &str,
        ttl: Duration,
        token_data: TokenData,
    ) -> Result<(), TokenError>;

    /// Clear all user tokens (token:* keys).
    async fn clear_all_user_tokens(&self) -> Result<(), TokenError>;

    /// Check if admin token exists.
    async fn admin_token_exists(&self) -> Result<bool, TokenError>;

    /// Get admin token hash.
    async fn get_admin_token(&self) -> Result<Option<String>, TokenError>;

    /// Store admin token hash.
    async fn store_admin_token(&self, token_hash: &str) -> Result<(), TokenError>;

    /// Count the number of active user tokens.
    async fn user_token_count(&self) -> Result<usize, TokenError>;
}
