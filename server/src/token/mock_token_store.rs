// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;

use super::{TokenData, TokenError, TokenStore};

/// Mock implementation of TokenStore trait with builder pattern.
///
/// This mock allows configuring various test scenarios including:
/// - Token count responses
/// - Store failures
/// - Admin token existence
/// - Custom token storage behavior
#[derive(Clone)]
pub struct MockTokenStore {
    /// Whether operations should fail
    should_fail: Arc<Mutex<bool>>,
    /// Admin token hash
    admin_token: Arc<Mutex<Option<String>>>,
    /// Stored tokens for retrieval
    stored_tokens: Arc<Mutex<HashMap<String, TokenData>>>,
}

impl MockTokenStore {
    /// Create a new mock token store
    pub fn new() -> Self {
        Self {
            should_fail: Arc::new(Mutex::new(false)),
            admin_token: Arc::new(Mutex::new(None)),
            stored_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Private accessor functions for cleaner lock handling

    fn should_fail(&self) -> bool {
        *self.should_fail.lock().expect("Failed to acquire lock")
    }

    fn set_should_fail_internal(&self, fail: bool) {
        *self.should_fail.lock().expect("Failed to acquire lock") = fail;
    }

    fn get_admin_token_internal(&self) -> Option<String> {
        self.admin_token
            .lock()
            .expect("Failed to acquire lock")
            .clone()
    }

    fn set_admin_token_internal(&self, token: Option<String>) {
        *self.admin_token.lock().expect("Failed to acquire lock") = token;
    }

    fn get_stored_tokens_mut(&self) -> std::sync::MutexGuard<'_, HashMap<String, TokenData>> {
        self.stored_tokens.lock().expect("Failed to acquire lock")
    }

    /// Configure operations to fail
    pub fn with_failures(self) -> Self {
        self.set_should_fail_internal(true);
        self
    }

    /// Set admin token
    pub fn with_admin_token(self, token_hash: &str) -> Self {
        self.set_admin_token_internal(Some(token_hash.to_string()));
        self
    }

    /// Add a stored token
    pub fn with_stored_token(self, token_hash: &str, token_data: TokenData) -> Self {
        self.get_stored_tokens_mut()
            .insert(token_hash.to_string(), token_data);
        self
    }

    /// Enable/disable failures (for testing error scenarios)
    pub fn set_should_fail(&self, should_fail: bool) {
        self.set_should_fail_internal(should_fail);
    }
}

impl Default for MockTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenStore for MockTokenStore {
    async fn get_token(&self, token_hash: &str) -> Result<Option<TokenData>, TokenError> {
        if self.should_fail() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(self.get_stored_tokens_mut().get(token_hash).cloned())
    }

    async fn store_token(
        &self,
        token_hash: &str,
        _ttl: Duration,
        token_data: TokenData,
    ) -> Result<(), TokenError> {
        if self.should_fail() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        self.get_stored_tokens_mut()
            .insert(token_hash.to_string(), token_data);
        Ok(())
    }

    async fn clear_all_user_tokens(&self) -> Result<(), TokenError> {
        if self.should_fail() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        self.get_stored_tokens_mut().clear();
        Ok(())
    }

    async fn admin_token_exists(&self) -> Result<bool, TokenError> {
        if self.should_fail() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(self.get_admin_token_internal().is_some())
    }

    async fn get_admin_token(&self) -> Result<Option<String>, TokenError> {
        if self.should_fail() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(self.get_admin_token_internal())
    }

    async fn store_admin_token(&self, token_hash: &str) -> Result<(), TokenError> {
        if self.should_fail() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        self.set_admin_token_internal(Some(token_hash.to_string()));
        Ok(())
    }

    async fn user_token_count(&self) -> Result<usize, TokenError> {
        if self.should_fail() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(self.get_stored_tokens_mut().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenData;

    #[tokio::test]
    async fn test_mock_token_store_builder() {
        let mock = MockTokenStore::new()
            .with_admin_token("admin_hash")
            .with_stored_token(
                "token_hash",
                TokenData {
                    upload_size_limit: Some(1024),
                },
            );

        // Test admin token
        assert!(
            mock.admin_token_exists()
                .await
                .expect("Failed to check for existing admin token")
        );
        assert_eq!(
            mock.get_admin_token()
                .await
                .expect("Failed to get admin token"),
            Some("admin_hash".to_string())
        );

        // Test stored token
        let token = mock
            .get_token("token_hash")
            .await
            .expect("Failed to get token");
        let token_data = token.expect("Token should exist");
        assert_eq!(token_data.upload_size_limit, Some(1024));

        let count = mock
            .user_token_count()
            .await
            .expect("Failed to get token count");
        assert!(count > 0, "Token count should be greater than 0");
    }

    #[tokio::test]
    async fn test_mock_token_store_failures() {
        let mock = MockTokenStore::new().with_failures();

        // All operations should fail
        let get_result = mock.get_token("any").await;
        assert!(
            get_result.is_err(),
            "Expected error for get_token, got: {:?}",
            get_result
        );
        let store_result = mock
            .store_token(
                "any",
                Duration::from_secs(3600),
                TokenData {
                    upload_size_limit: None,
                },
            )
            .await;
        assert!(
            store_result.is_err(),
            "Expected error for store_token, got: {:?}",
            store_result
        );
        let admin_result = mock.admin_token_exists().await;
        assert!(
            admin_result.is_err(),
            "Expected error for admin_token_exists, got: {:?}",
            admin_result
        );
    }

    #[tokio::test]
    async fn test_mock_token_store_clear_tokens() {
        let mock = MockTokenStore::new().with_stored_token(
            "token1",
            TokenData {
                upload_size_limit: None,
            },
        );

        // Verify initial state
        let count = mock
            .user_token_count()
            .await
            .expect("Failed to get initial token count");
        assert_eq!(count, 1, "Initial token count should be 1");

        // Clear tokens
        mock.clear_all_user_tokens()
            .await
            .expect("Failed to clear tokens");

        // Verify cleared state
        let count = mock
            .user_token_count()
            .await
            .expect("Failed to get final token count");
        assert_eq!(count, 0, "Token count after clear should be 0");
    }

    #[tokio::test]
    async fn test_mock_token_store_dynamic_updates() {
        let mock = MockTokenStore::new();

        mock.set_should_fail(true);
        let fail_result = mock.user_token_count().await;
        assert!(
            fail_result.is_err(),
            "Expected error when should_fail is true, got: {:?}",
            fail_result
        );

        mock.set_should_fail(false);
        assert_eq!(
            mock.user_token_count()
                .await
                .expect("Should succeed when should_fail is false"),
            0
        );
    }
}
