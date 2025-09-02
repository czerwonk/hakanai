// SPDX-License-Identifier: Apache-2.0

//! Test utilities for mocking token-related functionality.
//!
//! Provides a flexible mock implementation of TokenValidator and TokenCreator traits
//! with builder pattern for easy test configuration.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix_web::http::header::HeaderMap;
use async_trait::async_trait;
use uuid::Uuid;

use hakanai_lib::models::SecretRestrictions;

use crate::data_store::{DataStore, DataStoreError, DataStorePopResult};
use crate::observer::{SecretEventContext, SecretObserver};
use crate::token::{TokenCreator, TokenData, TokenError, TokenStore, TokenValidator};

/// Mock implementation of TokenValidator and TokenCreator traits with builder pattern.
///
/// This mock allows configuring various test scenarios including:
/// - Valid user tokens with associated data
/// - Valid admin tokens
/// - Token creation success/failure
/// - Custom token generation results
#[derive(Clone)]
pub struct MockTokenManager {
    /// Valid user tokens mapped to their metadata
    user_tokens: Arc<Mutex<HashMap<String, TokenData>>>,
    /// Valid admin tokens
    admin_tokens: Arc<Mutex<Vec<String>>>,
    /// Whether token creation should fail
    creation_should_fail: Arc<Mutex<bool>>,
    /// Token to return on successful creation
    created_token: Arc<Mutex<String>>,
}

impl MockTokenManager {
    /// Create a new mock token manager
    pub fn new() -> Self {
        Self {
            user_tokens: Arc::new(Mutex::new(HashMap::new())),
            admin_tokens: Arc::new(Mutex::new(Vec::new())),
            creation_should_fail: Arc::new(Mutex::new(false)),
            created_token: Arc::new(Mutex::new("mock_token".to_string())),
        }
    }

    /// Add a valid user token with associated metadata
    pub fn with_user_token(self, token: &str, data: TokenData) -> Self {
        self.user_tokens
            .lock()
            .unwrap()
            .insert(token.to_string(), data);
        self
    }

    /// Add a valid admin token
    pub fn with_admin_token(self, token: &str) -> Self {
        self.admin_tokens.lock().unwrap().push(token.to_string());
        self
    }

    /// Configure token creation to fail
    pub fn with_creation_failure(self) -> Self {
        *self.creation_should_fail.lock().unwrap() = true;
        self
    }

    /// Configure the token to return on successful creation
    pub fn with_created_token(self, token: &str) -> Self {
        *self.created_token.lock().unwrap() = token.to_string();
        self
    }

    /// Add multiple user tokens with unlimited upload size
    pub fn with_unlimited_user_tokens(self, tokens: &[&str]) -> Self {
        let mut user_tokens = self.user_tokens.lock().unwrap();
        for token in tokens {
            user_tokens.insert(
                token.to_string(),
                TokenData {
                    upload_size_limit: None,
                },
            );
        }
        drop(user_tokens);
        self
    }

    /// Add user token with specific upload size limit
    pub fn with_limited_user_token(self, token: &str, size_limit: i64) -> Self {
        self.user_tokens.lock().unwrap().insert(
            token.to_string(),
            TokenData {
                upload_size_limit: Some(size_limit),
            },
        );
        self
    }

    /// Add multiple admin tokens
    pub fn with_admin_tokens(self, tokens: &[&str]) -> Self {
        let mut admin_tokens = self.admin_tokens.lock().unwrap();
        for token in tokens {
            admin_tokens.push(token.to_string());
        }
        drop(admin_tokens);
        self
    }
}

impl Default for MockTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenValidator for MockTokenManager {
    async fn validate_user_token(&self, token: &str) -> Result<TokenData, TokenError> {
        let user_tokens = self.user_tokens.lock().unwrap();
        if let Some(data) = user_tokens.get(token) {
            Ok(data.clone())
        } else {
            Err(TokenError::InvalidToken)
        }
    }

    async fn validate_admin_token(&self, token: &str) -> Result<(), TokenError> {
        let admin_tokens = self.admin_tokens.lock().unwrap();
        if admin_tokens.contains(&token.to_string()) {
            Ok(())
        } else {
            Err(TokenError::InvalidToken)
        }
    }
}

#[async_trait]
impl TokenCreator for MockTokenManager {
    async fn create_user_token(
        &self,
        _token_data: TokenData,
        _ttl: Duration,
    ) -> Result<String, TokenError> {
        if *self.creation_should_fail.lock().unwrap() {
            Err(TokenError::Custom("Mock creation failure".to_string()))
        } else {
            Ok(self.created_token.lock().unwrap().clone())
        }
    }
}

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

    #[tokio::test]
    async fn test_mock_token_manager_builder() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockTokenManager::new()
            .with_user_token(
                "user_token",
                TokenData {
                    upload_size_limit: Some(1024),
                },
            )
            .with_admin_token("admin_token")
            .with_created_token("new_token");

        // Test user token validation
        let result = mock.validate_user_token("user_token").await?;
        assert_eq!(result.upload_size_limit, Some(1024));

        // Test invalid user token
        let result = mock.validate_user_token("invalid_token").await;
        assert!(
            result.is_err(),
            "Expected error for invalid user token, got: {:?}",
            result
        );

        // Test admin token validation
        mock.validate_admin_token("admin_token").await?;

        // Test invalid admin token
        let result = mock.validate_admin_token("invalid_admin").await;
        assert!(
            result.is_err(),
            "Expected error for invalid admin token, got: {:?}",
            result
        );

        // Test token creation
        let result = mock
            .create_user_token(
                TokenData {
                    upload_size_limit: None,
                },
                Duration::from_secs(3600),
            )
            .await?;
        assert_eq!(result, "new_token");

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_token_manager_creation_failure() {
        let mock = MockTokenManager::new().with_creation_failure();

        let result = mock
            .create_user_token(
                TokenData {
                    upload_size_limit: None,
                },
                Duration::from_secs(3600),
            )
            .await;
        assert!(
            result.is_err(),
            "Expected error for token creation with failures, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_mock_token_manager_bulk_methods() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockTokenManager::new()
            .with_unlimited_user_tokens(&["token1", "token2"])
            .with_admin_tokens(&["admin1", "admin2"]);

        // Test multiple user tokens
        mock.validate_user_token("token1").await?;
        mock.validate_user_token("token2").await?;

        // Test multiple admin tokens
        mock.validate_admin_token("admin1").await?;
        mock.validate_admin_token("admin2").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_token_manager_limited_user_token() -> Result<(), Box<dyn std::error::Error>>
    {
        let mock = MockTokenManager::new().with_limited_user_token("limited_token", 2048);

        let result = mock.validate_user_token("limited_token").await?;
        assert_eq!(result.upload_size_limit, Some(2048));

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_token_store_builder() {
        let mock = MockTokenStore::new()
            .with_token_count(5)
            .with_admin_token("admin_hash")
            .with_stored_token(
                "token_hash",
                TokenData {
                    upload_size_limit: Some(1024),
                },
            );

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
        let mock = MockTokenStore::new().with_token_count(5).with_stored_token(
            "token1",
            TokenData {
                upload_size_limit: None,
            },
        );

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

/// Mock implementation of DataStore trait for testing.
///
/// This mock allows configuring various test scenarios including:
/// - Secret count responses
/// - Store failures
/// - Custom secret storage behavior
#[derive(Clone)]
pub struct MockDataStore {
    /// Secret count to return
    secret_count: Arc<Mutex<usize>>,
    /// Whether operations should fail
    should_fail: Arc<Mutex<bool>>,
    /// Stored secrets for retrieval
    stored_secrets: Arc<Mutex<HashMap<String, String>>>,
    /// Secrets marked as accessed
    accessed_secrets: Arc<Mutex<Vec<String>>>,
    /// Custom pop result to return (for testing different scenarios)
    custom_pop_result: Arc<Mutex<Option<DataStorePopResult>>>,
    /// Track all put operations for testing verification
    put_operations: Arc<Mutex<Vec<(Uuid, String, Duration)>>>,
    /// Track all set_restrictions operations for testing verification
    set_restrictions_operations: Arc<Mutex<Vec<(Uuid, SecretRestrictions, Duration)>>>,
    /// Restrictions for secrets
    restrictions: Arc<Mutex<HashMap<String, SecretRestrictions>>>,
}

impl MockDataStore {
    /// Create a new mock data store
    pub fn new() -> Self {
        Self {
            secret_count: Arc::new(Mutex::new(0)),
            should_fail: Arc::new(Mutex::new(false)),
            stored_secrets: Arc::new(Mutex::new(HashMap::new())),
            accessed_secrets: Arc::new(Mutex::new(Vec::new())),
            custom_pop_result: Arc::new(Mutex::new(None)),
            put_operations: Arc::new(Mutex::new(Vec::new())),
            set_restrictions_operations: Arc::new(Mutex::new(Vec::new())),
            restrictions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set the secret count to return
    pub fn with_secret_count(self, count: usize) -> Self {
        *self.secret_count.lock().unwrap() = count;
        self
    }

    /// Configure operations to fail
    pub fn with_failures(self) -> Self {
        *self.should_fail.lock().unwrap() = true;
        self
    }

    /// Add a stored secret
    #[allow(dead_code)]
    pub fn with_stored_secret(self, id: Uuid, data: &str) -> Self {
        self.stored_secrets
            .lock()
            .unwrap()
            .insert(id.to_string(), data.to_string());
        self
    }

    /// Mark a secret as accessed
    #[allow(dead_code)]
    pub fn with_accessed_secret(self, id: Uuid) -> Self {
        self.accessed_secrets.lock().unwrap().push(id.to_string());
        self
    }

    /// Set the secret count manually
    #[allow(dead_code)]
    pub fn set_secret_count(&self, count: usize) {
        *self.secret_count.lock().unwrap() = count;
    }

    /// Set a custom pop result for testing specific scenarios
    pub fn with_pop_result(self, result: DataStorePopResult) -> Self {
        *self.custom_pop_result.lock().unwrap() = Some(result);
        self
    }

    /// Configure pop operations to fail
    pub fn with_get_error(self) -> Self {
        *self.should_fail.lock().unwrap() = true;
        self
    }

    /// Configure put operations to fail
    pub fn with_put_error(self) -> Self {
        *self.should_fail.lock().unwrap() = true;
        self
    }

    /// Get the stored secrets for testing verification
    #[allow(dead_code)]
    pub fn get_stored_secrets(&self) -> HashMap<String, String> {
        self.stored_secrets.lock().unwrap().clone()
    }

    /// Get all put operations for testing verification
    pub fn get_put_operations(&self) -> Vec<(Uuid, String, Duration)> {
        self.put_operations.lock().unwrap().clone()
    }

    /// Get all set_restrictions operations for testing verification
    pub fn get_set_restrictions_operations(&self) -> Vec<(Uuid, SecretRestrictions, Duration)> {
        self.set_restrictions_operations.lock().unwrap().clone()
    }

    /// Set restrictions for a secret (for testing)
    pub fn with_restrictions(self, id: Uuid, restrictions: SecretRestrictions) -> Self {
        self.restrictions
            .lock()
            .unwrap()
            .insert(id.to_string(), restrictions);
        self
    }

    /// Get all restrictions for testing verification
    #[allow(dead_code)]
    pub fn get_restrictions(&self) -> HashMap<String, SecretRestrictions> {
        self.restrictions.lock().unwrap().clone()
    }
}

impl Default for MockDataStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataStore for MockDataStore {
    async fn pop(&self, id: Uuid) -> Result<DataStorePopResult, DataStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DataStoreError::InternalError("Mock failure".to_string()));
        }

        // Check if we have a custom pop result
        if let Some(result) = self.custom_pop_result.lock().unwrap().as_ref() {
            return Ok(result.clone());
        }

        let id_str = id.to_string();

        // Check if secret exists
        if let Some(secret) = self.stored_secrets.lock().unwrap().remove(&id_str) {
            // Mark as accessed
            self.accessed_secrets.lock().unwrap().push(id_str);
            return Ok(DataStorePopResult::Found(secret));
        }

        // Check if already accessed
        if self.accessed_secrets.lock().unwrap().contains(&id_str) {
            return Ok(DataStorePopResult::AlreadyAccessed);
        }

        Ok(DataStorePopResult::NotFound)
    }

    async fn put(
        &self,
        id: Uuid,
        data: String,
        expires_in: Duration,
    ) -> Result<(), DataStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DataStoreError::InternalError("Mock failure".to_string()));
        }

        // Record the put operation for testing verification
        self.put_operations
            .lock()
            .unwrap()
            .push((id, data.clone(), expires_in));

        self.stored_secrets
            .lock()
            .unwrap()
            .insert(id.to_string(), data);

        // Update secret count
        let count = self.stored_secrets.lock().unwrap().len();
        *self.secret_count.lock().unwrap() = count;

        Ok(())
    }

    async fn is_healthy(&self) -> Result<(), DataStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DataStoreError::InternalError("Mock failure".to_string()));
        }
        Ok(())
    }

    async fn active_secret_count(&self) -> Result<usize, DataStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DataStoreError::InternalError("Mock failure".to_string()));
        }
        Ok(*self.secret_count.lock().unwrap())
    }

    async fn set_restrictions(
        &self,
        id: Uuid,
        restrictions: &SecretRestrictions,
        expires_in: Duration,
    ) -> Result<(), DataStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DataStoreError::InternalError("Mock failure".to_string()));
        }

        // Record the set_restrictions operation for testing verification
        self.set_restrictions_operations.lock().unwrap().push((
            id,
            restrictions.clone(),
            expires_in,
        ));

        // Store the restrictions
        if !restrictions.is_empty() {
            self.restrictions
                .lock()
                .unwrap()
                .insert(id.to_string(), restrictions.clone());
        }
        Ok(())
    }

    async fn get_restrictions(
        &self,
        id: Uuid,
    ) -> Result<Option<SecretRestrictions>, DataStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(DataStoreError::InternalError("Mock failure".to_string()));
        }

        // Retrieve the restrictions
        let restrictions = self
            .restrictions
            .lock()
            .unwrap()
            .get(&id.to_string())
            .cloned();

        Ok(restrictions)
    }
}

/// Mock implementation of SecretObserver trait for testing.
///
/// This mock allows tracking secret creation and retrieval events
/// for verification in tests.
#[derive(Clone)]
pub struct MockObserver {
    created_events: Arc<Mutex<Vec<(Uuid, HeaderMap)>>>,
    retrieved_events: Arc<Mutex<Vec<(Uuid, HeaderMap)>>>,
}

impl MockObserver {
    pub fn new() -> Self {
        MockObserver {
            created_events: Arc::new(Mutex::new(Vec::new())),
            retrieved_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_created_events(&self) -> Vec<(Uuid, HeaderMap)> {
        self.created_events.lock().unwrap().clone()
    }

    pub fn get_retrieved_events(&self) -> Vec<(Uuid, HeaderMap)> {
        self.retrieved_events.lock().unwrap().clone()
    }
}

#[async_trait]
impl SecretObserver for MockObserver {
    async fn on_secret_created(&self, secret_id: Uuid, context: &SecretEventContext) {
        self.created_events
            .lock()
            .unwrap()
            .push((secret_id, context.headers.clone()));
    }

    async fn on_secret_retrieved(&self, secret_id: Uuid, context: &SecretEventContext) {
        self.retrieved_events
            .lock()
            .unwrap()
            .push((secret_id, context.headers.clone()));
    }
}
