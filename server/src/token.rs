// SPDX-License-Identifier: Apache-2.0

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

use hakanai_lib::hash::sha256_hex_from_string;

const DEFAULT_TOKEN_TTL: u64 = 60 * 60 * 24 * 365; // 1 year in seconds

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

#[async_trait]
pub trait TokenValidator: Send + Sync {
    /// Validate token and return metadata.
    async fn validate_user_token(&self, token: &str) -> Result<TokenData, TokenError>;

    /// Validate admin token.
    async fn validate_admin_token(&self, token: &str) -> Result<(), TokenError>;
}

#[async_trait]
pub trait TokenCreator: Send + Sync {
    /// Create a new user token with specified metadata and TTL.
    async fn create_user_token(
        &self,
        token_data: TokenData,
        ttl: Duration,
    ) -> Result<String, TokenError>;
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
        if self.token_store.user_token_count().await? > 0 {
            return Ok(None);
        }

        self.create_default_token().await.map(Some)
    }

    /// Create default token (always creates new token).
    pub async fn create_default_token(&self) -> Result<String, TokenError> {
        let token_data = TokenData {
            upload_size_limit: None,
        };
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
        let token_hash = sha256_hex_from_string(&token);
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
        let token_hash = sha256_hex_from_string(&token);
        self.token_store.store_admin_token(&token_hash).await?;

        Ok(token)
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
    async fn validate_user_token(&self, token: &str) -> Result<TokenData, TokenError> {
        let token_hash = sha256_hex_from_string(token);

        match self.token_store.get_token(&token_hash).await? {
            Some(token_data) => Ok(token_data),
            None => Err(TokenError::InvalidToken),
        }
    }

    /// Validate admin token.
    async fn validate_admin_token(&self, token: &str) -> Result<(), TokenError> {
        let token_hash = sha256_hex_from_string(token);

        match self.token_store.get_admin_token().await? {
            Some(stored_hash) if stored_hash == token_hash => Ok(()),
            _ => Err(TokenError::InvalidToken),
        }
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
        let token_hash = sha256_hex_from_string(&token);
        self.token_store
            .store_token(&token_hash, ttl, token_data)
            .await?;

        Ok(token)
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
        admin_token: Arc<Mutex<Option<String>>>,
        should_fail: Arc<Mutex<bool>>,
    }

    impl MockTokenStore {
        fn new() -> Self {
            Self {
                tokens: Arc::new(Mutex::new(HashMap::new())),
                admin_token: Arc::new(Mutex::new(None)),
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

        async fn set_admin_token(&self, hash: &str) {
            *self.admin_token.lock().await = Some(hash.to_string());
        }
    }

    #[async_trait]
    impl TokenStore for MockTokenStore {
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

        async fn clear_all_user_tokens(&self) -> Result<(), TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }

            self.tokens.lock().await.clear();
            Ok(())
        }

        async fn admin_token_exists(&self) -> Result<bool, TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }

            Ok(self.admin_token.lock().await.is_some())
        }

        async fn get_admin_token(&self) -> Result<Option<String>, TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }

            Ok(self.admin_token.lock().await.clone())
        }

        async fn store_admin_token(&self, token_hash: &str) -> Result<(), TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }

            *self.admin_token.lock().await = Some(token_hash.to_string());
            Ok(())
        }

        async fn user_token_count(&self) -> Result<usize, TokenError> {
            if *self.should_fail.lock().await {
                return Err(TokenError::Custom("Mock failure".to_string()));
            }

            Ok(self.tokens.lock().await.len())
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
        assert_eq!(mock_store.user_token_count().await.unwrap(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_token_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        let test_token = "test_token_123";
        let test_hash = sha256_hex_from_string(test_token);
        let test_data = TokenData {
            upload_size_limit: Some(5000),
        };

        mock_store.insert_token(&test_hash, test_data.clone()).await;

        let token_data = manager.validate_user_token(test_token).await?;
        assert_eq!(token_data.upload_size_limit, Some(5000));
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_token_not_found() {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store);

        let result = manager.validate_user_token("nonexistent_token").await;

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

        let result = manager.validate_user_token("any_token").await;

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

    #[tokio::test]
    async fn test_validate_admin_token_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        let test_token = "admin_token_123";
        let test_hash = sha256_hex_from_string(test_token);
        mock_store.set_admin_token(&test_hash).await;

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
        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), TokenError::InvalidToken));
    }

    #[tokio::test]
    async fn test_validate_admin_token_wrong_hash() -> Result<(), Box<dyn std::error::Error>> {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        let correct_token = "admin_token_123";
        let wrong_token = "wrong_admin_token";
        let correct_hash = sha256_hex_from_string(correct_token);
        mock_store.set_admin_token(&correct_hash).await;

        let result = manager.validate_admin_token(wrong_token).await;
        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), TokenError::InvalidToken));
        Ok(())
    }

    #[tokio::test]
    async fn test_create_admin_token_success() -> Result<(), Box<dyn std::error::Error>> {
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
    async fn test_create_admin_token_if_none_when_empty() -> Result<(), Box<dyn std::error::Error>>
    {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store);

        let result = manager.create_admin_token_if_none().await?;
        assert!(result.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_create_admin_token_if_none_when_exists() -> Result<(), Box<dyn std::error::Error>>
    {
        let mock_store = MockTokenStore::new();
        let manager = TokenManager::new(mock_store.clone());

        // Pre-populate with an admin token
        mock_store.set_admin_token("existing_admin_hash").await;

        let result = manager.create_admin_token_if_none().await?;
        assert!(result.is_none());
        Ok(())
    }
}
