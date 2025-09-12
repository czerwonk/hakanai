use std::time::Duration;

use async_trait::async_trait;

use super::{TokenData, TokenError};

#[async_trait]
pub trait TokenCreator: Send + Sync {
    /// Create a new user token with specified metadata and TTL.
    async fn create_user_token(
        &self,
        token_data: TokenData,
        ttl: Duration,
    ) -> Result<String, TokenError>;
}
