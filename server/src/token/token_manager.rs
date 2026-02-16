use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use rand::TryRng;

use hakanai_lib::utils::hashing;

use super::{TokenCreator, TokenData, TokenError, TokenStore, TokenValidator};

const DEFAULT_TOKEN_TTL: u64 = 60 * 60 * 24 * 365; // 1 year in seconds

/// Manages token operations with hashing and validation.
#[derive(Clone)]
pub struct TokenManager<T: TokenStore> {
    token_store: T,
}

impl<T: TokenStore> TokenManager<T> {
    /// Create new token manager with storage backend.
    pub fn new(token_store: T) -> Self {
        Self { token_store }
    }

    /// Create default token if store is empty.
    pub async fn create_default_token_if_none(&self) -> Result<Option<String>, TokenError> {
        if self.token_store.user_token_count().await? > 0 {
            return Ok(None);
        }

        self.create_default_token().await.map(Some)
    }

    /// Create default token (always creates new token).
    pub async fn create_default_token(&self) -> Result<String, TokenError> {
        let token_data = TokenData::default();
        self.create_user_token(token_data, Duration::from_secs(DEFAULT_TOKEN_TTL))
            .await
    }

    /// Create a new user token with specified metadata and TTL.
    pub async fn create_user_token(
        &self,
        token_data: TokenData,
        ttl: Duration,
    ) -> Result<String, TokenError> {
        let token = Self::generate_token()?;
        let token_hash = hashing::sha256_hex_from_string(&token);
        self.token_store
            .store_token(&token_hash, ttl, token_data)
            .await?;

        Ok(token)
    }

    /// Clear all user tokens and create new default token.
    pub async fn reset_user_tokens(&self) -> Result<String, TokenError> {
        self.token_store.clear_all_user_tokens().await?;
        self.create_default_token().await
    }

    /// Create admin token if none exists.
    pub async fn create_admin_token_if_none(&self) -> Result<Option<String>, TokenError> {
        if self.token_store.admin_token_exists().await? {
            return Ok(None);
        }

        self.create_admin_token().await.map(Some)
    }

    /// Create admin token (always creates new token).
    pub async fn create_admin_token(&self) -> Result<String, TokenError> {
        let token = Self::generate_token()?;
        let token_hash = hashing::sha256_hex_from_string(&token);
        self.token_store.store_admin_token(&token_hash).await?;

        Ok(token)
    }

    /// Generate 32-byte cryptographically secure token.
    fn generate_token() -> Result<String, TokenError> {
        let mut bytes = [0u8; 32];

        let mut rng = rand::rng();
        if let Err(err) = rng.try_fill_bytes(&mut bytes) {
            return Err(TokenError::Custom(format!(
                "Failed to generate random bytes: {err}"
            )));
        }

        let token = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(bytes);
        Ok(token)
    }
}

#[async_trait]
impl<T: TokenStore> TokenCreator for TokenManager<T> {
    /// Create a new user token with specified metadata and TTL.
    async fn create_user_token(
        &self,
        token_data: TokenData,
        ttl: Duration,
    ) -> Result<String, TokenError> {
        let token = Self::generate_token()?;
        let token_hash = hashing::sha256_hex_from_string(&token);
        self.token_store
            .store_token(&token_hash, ttl, token_data)
            .await?;

        Ok(token)
    }
}

#[async_trait]
impl<T: TokenStore> TokenValidator for TokenManager<T> {
    /// Validate token and return metadata.
    async fn validate_user_token(&self, token: &str) -> Result<TokenData, TokenError> {
        let token_hash = hashing::sha256_hex_from_string(token);

        match self.token_store.get_token(&token_hash).await? {
            Some(token_data) => Ok(token_data),
            None => Err(TokenError::InvalidToken),
        }
    }

    /// Validate admin token.
    async fn validate_admin_token(&self, token: &str) -> Result<(), TokenError> {
        let token_hash = hashing::sha256_hex_from_string(token);

        match self.token_store.get_admin_token().await? {
            Some(stored_hash) if stored_hash == token_hash => Ok(()),
            _ => Err(TokenError::InvalidToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    use crate::token::MockTokenStore;

    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    #[tokio::test]
    async fn test_create_default_token_when_empty() -> Result<()> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        let result = manager.create_default_token_if_none().await?;
        assert!(result.is_some());

        // Verify token was stored
        let count = mock_store.user_token_count().await?;
        assert_eq!(count, 1);

        // Verify token is valid base64 URL-safe
        let token_str = result.ok_or("Token should not be None")?;
        base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(&token_str)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_no_token_created_when_not_empty() -> Result<()> {
        let mock_store = MockTokenStore::new().with_stored_token(
            "existing_hash",
            TokenData::default().with_upload_size_limit(1000),
        );
        let manager = TokenManager::new(mock_store.clone());

        let result = manager.create_default_token_if_none().await?;
        assert!(result.is_none());

        // Verify no additional token was created
        let count = mock_store.user_token_count().await?;
        assert_eq!(count, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_token_success() -> Result<()> {
        let test_token = "test_token_123";
        let test_hash = hashing::sha256_hex_from_string(test_token);
        let test_data = TokenData::default().with_upload_size_limit(5000);

        let mock_store = MockTokenStore::new().with_stored_token(&test_hash, test_data);
        let manager = TokenManager::new(mock_store.clone());

        let token_data = manager.validate_user_token(test_token).await?;
        assert_eq!(token_data.upload_size_limit, Some(5000));
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_token_not_found() {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store);

        let result = manager.validate_user_token("nonexistent_token").await;

        assert!(
            result.is_err(),
            "Expected error for nonexistent token, got: {:?}",
            result
        );
        assert!(matches!(result.unwrap_err(), TokenError::InvalidToken));
    }

    #[tokio::test]
    async fn test_generate_token_format() -> Result<()> {
        let token = TokenManager::<MockTokenStore>::generate_token()?;

        // Should be base64 URL-safe encoded
        let decoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(&token)?;

        // Should be 32 bytes = ~43 characters when base64 encoded
        assert_eq!(decoded.len(), 32);
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_token_uniqueness() -> Result<()> {
        let token1 = TokenManager::<MockTokenStore>::generate_token()?;
        let token2 = TokenManager::<MockTokenStore>::generate_token()?;

        assert_ne!(token1, token2);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_default_token_store_failure() {
        let mock_store = MockTokenStore::new().with_failures();
        let manager = TokenManager::new(mock_store);

        let result = manager.create_default_token_if_none().await;

        assert!(
            result.is_err(),
            "Expected error for store failure, got: {:?}",
            result
        );
        assert!(matches!(result.unwrap_err(), TokenError::Custom(_)));
    }

    #[tokio::test]
    async fn test_validate_token_store_failure() {
        let mock_store = MockTokenStore::new().with_failures();
        let manager = TokenManager::new(mock_store);

        let result = manager.validate_user_token("any_token").await;

        assert!(
            result.is_err(),
            "Expected error for store failure, got: {:?}",
            result
        );
        assert!(matches!(result.unwrap_err(), TokenError::Custom(_)));
    }

    #[tokio::test]
    async fn test_validate_admin_token_success() -> Result<()> {
        let test_token = "admin_token_123";
        let test_hash = hashing::sha256_hex_from_string(test_token);

        let mock_store = MockTokenStore::new().with_admin_token(&test_hash);
        let manager = TokenManager::new(mock_store.clone());

        manager.validate_admin_token(test_token).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_admin_token_not_found() {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store);

        let result = manager
            .validate_admin_token("nonexistent_admin_token")
            .await;
        assert!(
            result.is_err(),
            "Expected error for nonexistent admin token, got: {:?}",
            result
        );
        assert!(matches!(result.unwrap_err(), TokenError::InvalidToken));
    }

    #[tokio::test]
    async fn test_validate_admin_token_wrong_hash() -> Result<()> {
        let correct_token = "admin_token_123";
        let wrong_token = "wrong_admin_token";
        let correct_hash = hashing::sha256_hex_from_string(correct_token);

        let mock_store = MockTokenStore::new().with_admin_token(&correct_hash);
        let manager = TokenManager::new(mock_store.clone());

        let result = manager.validate_admin_token(wrong_token).await;
        assert!(
            result.is_err(),
            "Expected error for wrong admin token, got: {:?}",
            result
        );
        assert!(matches!(result.unwrap_err(), TokenError::InvalidToken));
        Ok(())
    }

    #[tokio::test]
    async fn test_create_admin_token_success() -> Result<()> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        let token = manager.create_admin_token().await?;

        // Verify token is valid base64 URL-safe
        base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(&token)?;

        // Verify token can be validated
        manager.validate_admin_token(&token).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_admin_token_if_none_when_empty() -> Result<()> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store);

        let result = manager.create_admin_token_if_none().await?;
        assert!(result.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_create_admin_token_if_none_when_exists() -> Result<()> {
        let mock_store = MockTokenStore::new().with_admin_token("existing_admin_hash");
        let manager = TokenManager::new(mock_store.clone());

        let result = manager.create_admin_token_if_none().await?;
        assert!(result.is_none());
        Ok(())
    }
}
