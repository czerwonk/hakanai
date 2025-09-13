// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

use hakanai_lib::models::SecretRestrictions;

/// `SecretStoreError` is an enum that represents the possible errors that can occur when accessing secret storage.
/// It implements the `std::error::Error` trait and can be used to handle errors in a consistent way across the application.
#[derive(Debug, Error)]
pub enum SecretStoreError {
    /// Represents an error that occurs when accessing the Redis data store (implementation
    /// Specific).
    #[error("data store access error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Represents an error when the current timestamp cannot be retrieved. This should not happen.
    #[error("cold not get current timestamp: {0}")]
    TimestampError(#[from] std::time::SystemTimeError),

    /// Internal error while accessing the data store. This is only used in tests.
    #[error("internal error: {0}")]
    #[cfg(test)]
    InternalError(String),

    #[error("error while JSON processing: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// `SecretStorePopResult` is an enum that represents the possible outcomes of SecretStore::pop operation.
#[derive(Debug, Clone)]
pub enum SecretStorePopResult {
    /// Represents a successful retrieval of data from the data store.
    Found(String),

    /// Represents a case where the requested data was not found in the data store.
    NotFound,

    /// Represents a case where the data store was accessed before and does not exist anymore.
    AlreadyAccessed,
}

/// `SecretStore` is a trait that defines the contract for a simple, asynchronous,
/// key-value storage system. Implementations of this trait are expected to be
/// thread-safe.
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Atomically retrieves and removes a value from the data store based on its
    /// `Uuid`.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the item to retrieve and remove.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(SecretStorePopResult)` if no unexpected error occured.
    /// If the item was found, it returns `SecretStorePopResult::Found(String)`.
    /// If the item was not found, it returns `SecretStorePopResult::NotFound`.
    /// If the item was accessed before and does not exist anymore, it returns SecretStorePopResult::AlreadyAccessed.
    /// If an error occurs, it returns `SecretStoreError`.
    async fn pop(&self, id: Uuid) -> Result<SecretStorePopResult, SecretStoreError>;

    /// Stores a value in the data store with a given `Uuid` and an expiration
    /// duration.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` to use as the key for the stored data.
    /// * `data` - The `String` data to store.
    /// * `expires_in` - A `Duration` after which the stored item should be
    ///   considered expired. Note that the implementation of the data store
    ///   determines how expiration is handled.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` on successful insertion, or an `Err` if an
    /// error occurs.
    async fn put(
        &self,
        id: Uuid,
        data: String,
        expires_in: Duration,
    ) -> Result<(), SecretStoreError>;

    /// Checks if the data store is healthy and can be accessed.
    ///
    /// # Returns
    /// true if the data store is healthy, false otherwise.
    async fn is_healthy(&self) -> Result<(), SecretStoreError>;

    /// Stores IP restrictions for a secret with the same TTL as the secret itself.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the secret.
    /// * `allowed_ips` - Vector of IP networks that are allowed to access this secret.
    /// * `expires_in` - The duration after which the restrictions should expire.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` on successful storage, or an `Err` if an error occurs.
    async fn set_restrictions(
        &self,
        id: Uuid,
        restrictions: &SecretRestrictions,
        expires_in: Duration,
    ) -> Result<(), SecretStoreError>;

    /// Retrieves access restrictions for a secret (if any).
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the secret.
    ///
    /// # Returns
    ///
    /// A `Result` containing `Some(SecretRestrictions)` if restrictions exist,
    /// `None` if no restrictions, or an `Err` if an error occurs.
    async fn get_restrictions(
        &self,
        id: Uuid,
    ) -> Result<Option<SecretRestrictions>, SecretStoreError>;
}
