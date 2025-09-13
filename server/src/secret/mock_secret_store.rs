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
    pub fn with_pop_result(self, result: SecretStorePopResult) -> Self {
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

impl Default for MockSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretStore for MockSecretStore {
    async fn pop(&self, id: Uuid) -> Result<SecretStorePopResult, SecretStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
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
            return Ok(SecretStorePopResult::Found(secret));
        }

        // Check if already accessed
        if self.accessed_secrets.lock().unwrap().contains(&id_str) {
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
        if *self.should_fail.lock().unwrap() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
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

    async fn is_healthy(&self) -> Result<(), SecretStoreError> {
        if *self.should_fail.lock().unwrap() {
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
        if *self.should_fail.lock().unwrap() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
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
    ) -> Result<Option<SecretRestrictions>, SecretStoreError> {
        if *self.should_fail.lock().unwrap() {
            return Err(SecretStoreError::InternalError("Mock failure".to_string()));
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
