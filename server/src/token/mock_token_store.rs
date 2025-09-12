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
    /// Token count to return
    token_count: Arc<Mutex<usize>>,
    /// Whether operations should fail
    should_fail: Arc<Mutex<bool>>,
    /// Whether store is empty
    is_empty: Arc<Mutex<bool>>,
    /// Admin token hash
    admin_token: Arc<Mutex<Option<String>>>,
    /// Stored tokens for retrieval
    stored_tokens: Arc<Mutex<HashMap<String, TokenData>>>,
}

impl MockTokenStore {
    /// Create a new mock token store
    pub fn new() -> Self {
        Self {
            token_count: Arc::new(Mutex::new(0)),
            should_fail: Arc::new(Mutex::new(false)),
            is_empty: Arc::new(Mutex::new(true)),
            admin_token: Arc::new(Mutex::new(None)),
            stored_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set the token count to return
    pub fn with_token_count(self, count: usize) -> Self {
        *self.token_count.lock().unwrap() = count;
        *self.is_empty.lock().unwrap() = count == 0;
        self
    }

    /// Configure operations to fail
    pub fn with_failures(self) -> Self {
        *self.should_fail.lock().unwrap() = true;
        self
    }

    /// Configure operations to succeed
    #[allow(dead_code)]
    pub fn with_success(self) -> Self {
        *self.should_fail.lock().unwrap() = false;
        self
    }

    /// Set admin token
    pub fn with_admin_token(self, token_hash: &str) -> Self {
        *self.admin_token.lock().unwrap() = Some(token_hash.to_string());
        self
    }

    /// Add a stored token
    pub fn with_stored_token(self, token_hash: &str, token_data: TokenData) -> Self {
        self.stored_tokens
            .lock()
            .unwrap()
            .insert(token_hash.to_string(), token_data);
        self.inc_token_count();
        self
    }

    /// Set store as empty
    pub fn with_empty_store(self) -> Self {
        *self.is_empty.lock().unwrap() = true;
        *self.token_count.lock().unwrap() = 0;
        self
    }

    /// Set store as non-empty
    pub fn with_non_empty_store(self, count: usize) -> Self {
        *self.is_empty.lock().unwrap() = false;
        *self.token_count.lock().unwrap() = count;
        self
    }

    /// Manually set the token count (for testing metrics)
    pub fn set_token_count(&self, count: usize) {
        *self.token_count.lock().unwrap() = count;
    }

    pub fn inc_token_count(&self) {
        let current = *self.token_count.lock().unwrap();
        *self.token_count.lock().unwrap() = current + 1;
    }

    /// Enable/disable failures (for testing error scenarios)
    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
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
        if *self.should_fail.lock().unwrap() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(self.stored_tokens.lock().unwrap().get(token_hash).cloned())
    }

    async fn store_token(
        &self,
        token_hash: &str,
        _ttl: Duration,
        token_data: TokenData,
    ) -> Result<(), TokenError> {
        if *self.should_fail.lock().unwrap() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        self.stored_tokens
            .lock()
            .unwrap()
            .insert(token_hash.to_string(), token_data);
        self.inc_token_count();
        Ok(())
    }

    async fn clear_all_user_tokens(&self) -> Result<(), TokenError> {
        if *self.should_fail.lock().unwrap() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        self.stored_tokens.lock().unwrap().clear();
        *self.token_count.lock().unwrap() = 0;
        *self.is_empty.lock().unwrap() = true;
        Ok(())
    }

    async fn admin_token_exists(&self) -> Result<bool, TokenError> {
        if *self.should_fail.lock().unwrap() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(self.admin_token.lock().unwrap().is_some())
    }

    async fn get_admin_token(&self) -> Result<Option<String>, TokenError> {
        if *self.should_fail.lock().unwrap() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(self.admin_token.lock().unwrap().clone())
    }

    async fn store_admin_token(&self, token_hash: &str) -> Result<(), TokenError> {
        if *self.should_fail.lock().unwrap() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        *self.admin_token.lock().unwrap() = Some(token_hash.to_string());
        Ok(())
    }

    async fn user_token_count(&self) -> Result<usize, TokenError> {
        if *self.should_fail.lock().unwrap() {
            return Err(TokenError::Custom("Mock failure".to_string()));
        }
        Ok(*self.token_count.lock().unwrap())
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
            )
            .with_token_count(5);

        // Test token count
        let count = mock.user_token_count().await.unwrap();
        assert_eq!(count, 5);

        // Test admin token
        assert!(mock.admin_token_exists().await.unwrap());
        assert_eq!(
            mock.get_admin_token().await.unwrap(),
            Some("admin_hash".to_string())
        );

        // Test stored token
        let token_data = mock.get_token("token_hash").await.unwrap().unwrap();
        assert_eq!(token_data.upload_size_limit, Some(1024));

        // Test is_empty (token count > 0)
        assert!(mock.user_token_count().await.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_mock_token_store_failures() {
        let mock = MockTokenStore::new().with_failures();

        // All operations should fail
        let count_result = mock.user_token_count().await;
        assert!(
            count_result.is_err(),
            "Expected error for user_token_count, got: {:?}",
            count_result
        );
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
    async fn test_mock_token_store_empty_store() {
        let mock = MockTokenStore::new().with_empty_store();

        assert_eq!(mock.user_token_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_mock_token_store_non_empty_store() {
        let mock = MockTokenStore::new().with_non_empty_store(10);

        assert!(mock.user_token_count().await.unwrap() > 0);
        assert_eq!(mock.user_token_count().await.unwrap(), 10);
    }

    #[tokio::test]
    async fn test_mock_token_store_clear_tokens() {
        let mock = MockTokenStore::new()
            .with_stored_token(
                "token1",
                TokenData {
                    upload_size_limit: None,
                },
            )
            .with_token_count(5);

        // Verify initial state
        assert_eq!(mock.user_token_count().await.unwrap(), 5);
        assert!(mock.get_token("token1").await.unwrap().is_some());

        // Clear tokens
        mock.clear_all_user_tokens().await.unwrap();

        // Verify cleared state
        assert_eq!(mock.user_token_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_mock_token_store_dynamic_updates() {
        let mock = MockTokenStore::new();

        // Test dynamic updates
        mock.set_token_count(15);
        assert_eq!(mock.user_token_count().await.unwrap(), 15);

        mock.set_should_fail(true);
        let fail_result = mock.user_token_count().await;
        assert!(
            fail_result.is_err(),
            "Expected error when should_fail is true, got: {:?}",
            fail_result
        );

        mock.set_should_fail(false);
        assert_eq!(mock.user_token_count().await.unwrap(), 15);
    }
}
