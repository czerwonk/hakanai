//! Token management for authentication and authorization.
//!
//! Provides token generation, validation, and storage abstraction.
//! Tokens are SHA-256 hashed before storage for security.

use core::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::hash::hash_string;

const DEFAULT_TOKEN_TTL: u64 = 60 * 60 * 24 * 30; // 30 days in seconds

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

/// Token metadata stored in Redis.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TokenData {
    /// Optional upload size limit in bytes.
    pub upload_size_limit: Option<i64>,
}

impl TokenData {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn deserialize(str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(str)
    }
}

/// Abstraction for token storage operations.
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Check if token store is empty.
    async fn is_empty(&self) -> Result<bool, TokenError>;

    /// Gets token metadata by its hash.
    async fn get_token(&self, token_hash: &str) -> Result<Option<TokenData>, TokenError>;

    /// Store token with metadata.
    async fn store_token(
        &self,
        token_hash: &str,
        ttl: Duration,
        token_data: TokenData,
    ) -> Result<(), TokenError>;
}

#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Validate token and return metadata.
    async fn validate_token(&self, token: &str) -> Result<TokenData, TokenError>;
}

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
        if !self.token_store.is_empty().await? {
            return Ok(None);
        }

        let token = Self::generate_token()?;
        let token_hash = hash_string(&token);
        self.token_store
            .store_token(
                &token_hash,
                Duration::from_secs(DEFAULT_TOKEN_TTL),
                TokenData {
                    upload_size_limit: None,
                },
            )
            .await?;

        Ok(Some(token))
    }

    /// Generate 32-byte cryptographically secure token.
    fn generate_token() -> Result<String, TokenError> {
        let mut bytes = [0u8; 32];

        if let Err(err) = OsRng.try_fill_bytes(&mut bytes) {
            return Err(TokenError::Custom(format!(
                "Failed to generate random bytes: {err}"
            )));
        }

        let token = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(bytes);
        Ok(token)
    }
}

#[async_trait]
impl<T: TokenStore> TokenValidator for TokenManager<T> {
    /// Validate token and return metadata.
    async fn validate_token(&self, token: &str) -> Result<TokenData, TokenError> {
        let token_hash = hash_string(token);

        match self.token_store.get_token(&token_hash).await? {
            Some(token_data) => Ok(token_data),
            None => Err(TokenError::InvalidToken),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock TokenStore for testing
    #[derive(Clone)]
    struct MockTokenStore {
        tokens: Arc<Mutex<HashMap<String, TokenData>>>,
        should_fail: Arc<Mutex<bool>>,
    }

    impl MockTokenStore {
        fn new() -> Self {
            Self {
                tokens: Arc::new(Mutex::new(HashMap::new())),
                should_fail: Arc::new(Mutex::new(false)),
            }
        }

        async fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().await = should_fail;
        }

        async fn insert_token(&self, hash: &str, data: TokenData) {
            self.tokens.lock().await.insert(hash.to_string(), data);
        }

        async fn token_count(&self) -> usize {
            self.tokens.lock().await.len()
        }
    }

    #[async_trait]
    impl TokenStore for MockTokenStore {
        async fn is_empty(&self) -> Result<bool, TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }
            Ok(self.tokens.lock().await.is_empty())
        }

        async fn get_token(&self, token_hash: &str) -> Result<Option<TokenData>, TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }

            Ok(self.tokens.lock().await.get(token_hash).cloned())
        }

        async fn store_token(
            &self,
            token_hash: &str,
            _ttl: Duration,
            token_data: TokenData,
        ) -> Result<(), TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }

            self.tokens
                .lock()
                .await
                .insert(token_hash.to_string(), token_data);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_default_token_when_empty() -> Result<(), Box<dyn std::error::Error>> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        let result = manager.create_default_token_if_none().await?;
        assert!(result.is_some());

        // Verify token was stored
        assert_eq!(mock_store.token_count().await, 1);

        // Verify token is valid base64 URL-safe
        let token_str = result.ok_or("Token should not be None")?;
        base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(&token_str)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_no_token_created_when_not_empty() -> Result<(), Box<dyn std::error::Error>> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        // Pre-populate with a token
        mock_store
            .insert_token(
                "existing_hash",
                TokenData {
                    upload_size_limit: Some(1000),
                },
            )
            .await;

        let result = manager.create_default_token_if_none().await?;
        assert!(result.is_none());

        // Verify no additional token was created
        assert_eq!(mock_store.token_count().await, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_token_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        let test_token = "test_token_123";
        let test_hash = hash_string(test_token);
        let test_data = TokenData {
            upload_size_limit: Some(5000),
        };

        mock_store.insert_token(&test_hash, test_data.clone()).await;

        let token_data = manager.validate_token(test_token).await?;
        assert_eq!(token_data.upload_size_limit, Some(5000));
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_token_not_found() {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store);

        let result = manager.validate_token("nonexistent_token").await;

        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), TokenError::InvalidToken));
    }

    #[tokio::test]
    async fn test_generate_token_format() -> Result<(), Box<dyn std::error::Error>> {
        let token = TokenManager::<MockTokenStore>::generate_token()?;

        // Should be base64 URL-safe encoded
        let decoded = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(&token)?;

        // Should be 32 bytes = ~43 characters when base64 encoded
        assert_eq!(decoded.len(), 32);
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_token_uniqueness() -> Result<(), Box<dyn std::error::Error>> {
        let token1 = TokenManager::<MockTokenStore>::generate_token()?;
        let token2 = TokenManager::<MockTokenStore>::generate_token()?;

        assert_ne!(token1, token2);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_default_token_store_failure() {
        let mock_store = MockTokenStore::new();
        mock_store.set_should_fail(true).await;
        let manager = TokenManager::new(mock_store);

        let result = manager.create_default_token_if_none().await;

        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), TokenError::Custom(_)));
    }

    #[tokio::test]
    async fn test_validate_token_store_failure() {
        let mock_store = MockTokenStore::new();
        mock_store.set_should_fail(true).await;
        let manager = TokenManager::new(mock_store);

        let result = manager.validate_token("any_token").await;

        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), TokenError::Custom(_)));
    }

    #[tokio::test]
    async fn test_token_data_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let token_data = TokenData {
            upload_size_limit: Some(1024),
        };

        // Test serialization
        let serialized = serde_json::to_string(&token_data)?;
        assert!(serialized.contains("1024"));

        // Test deserialization
        let deserialized: TokenData = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.upload_size_limit, Some(1024));
        Ok(())
    }

    #[tokio::test]
    async fn test_token_data_none_upload_limit() -> Result<(), Box<dyn std::error::Error>> {
        let token_data = TokenData {
            upload_size_limit: None,
        };

        let serialized = serde_json::to_string(&token_data)?;
        let deserialized: TokenData = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized.upload_size_limit, None);
        Ok(())
    }
}
