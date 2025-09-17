// SPDX-License-Identifier: Apache-2.0

//! Test utilities for mocking token-related functionality.
//!
//! Provides a flexible mock implementation of TokenValidator and TokenCreator traits
//! with builder pattern for easy test configuration.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use uuid::Uuid;

use hakanai_lib::models::SecretRestrictions;

use super::{SecretStore, SecretStoreError, SecretStorePopResult};

/// Mock implementation of SecretStore trait for testing.
///
/// This mock allows configuring various test scenarios including:
/// - Secret count responses
/// - Store failures
/// - Custom secret storage behavior
#[derive(Clone)]
pub struct MockSecretStore {
    /// Secret count to return
    secret_count: Arc<Mutex<usize>>,
    /// Whether operations should fail
    should_fail: Arc<Mutex<bool>>,
    /// Stored secrets for retrieval
    stored_secrets: Arc<Mutex<HashMap<String, String>>>,
    /// Secrets marked as accessed
    accessed_secrets: Arc<Mutex<Vec<String>>>,
    /// Custom pop result to return (for testing different scenarios)
    custom_pop_result: Arc<Mutex<Option<SecretStorePopResult>>>,
    /// Track all put operations for testing verification
    put_operations: Arc<Mutex<Vec<(Uuid, String, Duration)>>>,
    /// Track all set_restrictions operations for testing verification
    set_restrictions_operations: Arc<Mutex<Vec<(Uuid, SecretRestrictions, Duration)>>>,
    /// Restrictions for secrets
    restrictions: Arc<Mutex<HashMap<String, SecretRestrictions>>>,
}

impl MockSecretStore {
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

    // Private accessor functions for cleaner lock handling
    fn should_fail(&self) -> bool {
        *self.should_fail.lock().expect("Failed to acquire lock")
    }

    fn _get_secret_count(&self) -> usize {
        *self.secret_count.lock().expect("Failed to acquire lock")
    }

    fn set_secret_count_internal(&self, count: usize) {
        *self.secret_count.lock().expect("Failed to acquire lock") = count;
    }

    fn get_stored_secrets_mut(&self) -> std::sync::MutexGuard<'_, HashMap<String, String>> {
        self.stored_secrets.lock().expect("Failed to acquire lock")
    }

    fn get_accessed_secrets_mut(&self) -> std::sync::MutexGuard<'_, Vec<String>> {
        self.accessed_secrets
            .lock()
            .expect("Failed to acquire lock")
    }

    fn get_custom_pop_result(&self) -> Option<SecretStorePopResult> {
        self.custom_pop_result
            .lock()
            .expect("Failed to acquire lock")
            .clone()
    }

    fn set_custom_pop_result(&self, result: Option<SecretStorePopResult>) {
        *self
            .custom_pop_result
            .lock()
            .expect("Failed to acquire lock") = result;
    }

    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().expect("Failed to acquire lock") = fail;
    }

    fn get_put_operations_mut(&self) -> std::sync::MutexGuard<'_, Vec<(Uuid, String, Duration)>> {
        self.put_operations.lock().expect("Failed to acquire lock")
    }

    fn get_set_restrictions_operations_mut(
        &self,
    ) -> std::sync::MutexGuard<'_, Vec<(Uuid, SecretRestrictions, Duration)>> {
        self.set_restrictions_operations
            .lock()
            .expect("Failed to acquire lock")
    }

    fn get_restrictions_mut(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<String, SecretRestrictions>> {
        self.restrictions.lock().expect("Failed to acquire lock")
    }

    /// Set a custom pop result for testing specific scenarios
    pub fn with_pop_result(self, result: SecretStorePopResult) -> Self {
        self.set_custom_pop_result(Some(result));
        self
    }

    /// Configure pop operations to fail
    pub fn with_get_error(self) -> Self {
        self.set_should_fail(true);
        self
    }

    /// Configure put operations to fail
    pub fn with_put_error(self) -> Self {
        self.set_should_fail(true);
        self
    }

    /// Get all put operations for testing verification
    pub fn get_put_operations(&self) -> Vec<(Uuid, String, Duration)> {
        self.get_put_operations_mut().clone()
    }

    /// Get all set_restrictions operations for testing verification
    pub fn get_set_restrictions_operations(&self) -> Vec<(Uuid, SecretRestrictions, Duration)> {
        self.get_set_restrictions_operations_mut().clone()
    }

    /// Set restrictions for a secret (for testing)
    pub fn with_restrictions(self, id: Uuid, restrictions: SecretRestrictions) -> Self {
        self.get_restrictions_mut()
            .insert(id.to_string(), restrictions);
        self
    }

    /// Get all restrictions for testing verification
    pub fn get_restrictions(&self) -> HashMap<String, SecretRestrictions> {
        self.get_restrictions_mut().clone()
    }
}

impl Default for MockSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretStore for MockSecretStore {
    async fn pop(&self, id: Uuid) -> Result<SecretStorePopResult, SecretStoreError> {
        if self.should_fail() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
        }

        // Check if we have a custom pop result
        if let Some(result) = self.get_custom_pop_result() {
            return Ok(result);
        }

        let id_str = id.to_string();

        // Check if secret exists
        if let Some(secret) = self.get_stored_secrets_mut().remove(&id_str) {
            // Mark as accessed
            self.get_accessed_secrets_mut().push(id_str);
            return Ok(SecretStorePopResult::Found(secret));
        }

        // Check if already accessed
        if self.get_accessed_secrets_mut().contains(&id_str) {
            return Ok(SecretStorePopResult::AlreadyAccessed);
        }

        Ok(SecretStorePopResult::NotFound)
    }

    async fn put(
        &self,
        id: Uuid,
        data: String,
        expires_in: Duration,
    ) -> Result<(), SecretStoreError> {
        if self.should_fail() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
        }

        // Record the put operation for testing verification
        self.get_put_operations_mut()
            .push((id, data.clone(), expires_in));

        self.get_stored_secrets_mut().insert(id.to_string(), data);

        // Update secret count
        let count = self.get_stored_secrets_mut().len();
        self.set_secret_count_internal(count);

        Ok(())
    }

    async fn is_healthy(&self) -> Result<(), SecretStoreError> {
        if self.should_fail() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
        }
        Ok(())
    }

    async fn set_restrictions(
        &self,
        id: Uuid,
        restrictions: &SecretRestrictions,
        expires_in: Duration,
    ) -> Result<(), SecretStoreError> {
        if self.should_fail() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
        }

        // Record the set_restrictions operation for testing verification
        self.get_set_restrictions_operations_mut()
            .push((id, restrictions.clone(), expires_in));

        // Store the restrictions
        if !restrictions.is_empty() {
            self.get_restrictions_mut()
                .insert(id.to_string(), restrictions.clone());
        }
        Ok(())
    }

    async fn get_restrictions(
        &self,
        id: Uuid,
    ) -> Result<Option<SecretRestrictions>, SecretStoreError> {
        if self.should_fail() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
        }

        // Retrieve the restrictions
        let restrictions = self.get_restrictions_mut().get(&id.to_string()).cloned();

        Ok(restrictions)
    }
}
