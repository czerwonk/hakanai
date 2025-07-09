use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use thiserror::Error;
use tracing::{error, instrument};
use uuid::Uuid;

use hakanai_lib::timestamp;

/// `DataStoreError` is an enum that represents the possible errors that can occur when accessing
/// the data store.
/// It implements the `std::error::Error` trait and can be used to handle errors in a consistent way across the application.
#[derive(Debug, Error)]
pub enum DataStoreError {
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
}

/// `DataStorePopResult` is an enum that represents the possible outcomes of DataStore::pop operation.
#[derive(Debug, Clone)]
pub enum DataStorePopResult {
    /// Represents a successful retrieval of data from the data store.
    Found(String),

    /// Represents a case where the requested data was not found in the data store.
    NotFound,

    /// Represents a case where the data store was accessed before and does not exist anymore.
    AlreadyAccessed,
}

/// `DataStore` is a trait that defines the contract for a simple, asynchronous,
/// key-value storage system. Implementations of this trait are expected to be
/// thread-safe.
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Atomically retrieves and removes a value from the data store based on its
    /// `Uuid`.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the item to retrieve and remove.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(DataStorePopResult)` if no unexpected error occured.
    /// If the item was found, it returns `DataStorePopResult::Found(String)`.
    /// If the item was not found, it returns `DataStorePopResult::NotFound`.
    /// If the item was accessed before and does not exist anymore, it returns DataStorePopResult::AlreadyAccessed.
    /// If an error occurs, it returns `DataStoreError`.
    async fn pop(&self, id: Uuid) -> Result<DataStorePopResult, DataStoreError>;

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
    async fn put(&self, id: Uuid, data: String, expires_in: Duration)
    -> Result<(), DataStoreError>;
}

/// An implementation of the `DataStore` trait that uses Redis as its backend.
/// This struct holds a `ConnectionManager` for interacting with the Redis
/// server. It is designed to be cloneable and thread-safe.
#[derive(Clone)]
pub struct RedisDataStore {
    con: ConnectionManager,
    max_ttl: Duration,
}

impl RedisDataStore {
    pub async fn new(redis_url: &str, max_ttl: Duration) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let con = ConnectionManager::new(client).await?;
        Ok(Self { con, max_ttl })
    }
}

#[async_trait]
impl DataStore for RedisDataStore {
    #[instrument(skip(self), err)]
    async fn pop(&self, id: Uuid) -> Result<DataStorePopResult, DataStoreError> {
        let value = self.con.clone().get_del(id.to_string()).await?;

        if let Some(secret) = value {
            self.mark_as_accessed(id).await?;
            return Ok(DataStorePopResult::Found(secret));
        }

        if self.was_accessed(id).await? {
            return Ok(DataStorePopResult::AlreadyAccessed);
        }

        Ok(DataStorePopResult::NotFound)
    }

    #[instrument(skip(self, data), err)]
    async fn put(
        &self,
        id: Uuid,
        data: String,
        expires_in: Duration,
    ) -> Result<(), DataStoreError> {
        let _: () = self
            .con
            .clone()
            .set_ex(id.to_string(), data, expires_in.as_secs())
            .await?;
        Ok(())
    }
}

impl RedisDataStore {
    fn accessed_key(&self, id: Uuid) -> String {
        format!("accessed:{id}")
    }

    #[instrument(skip(self), err)]
    async fn was_accessed(&self, id: Uuid) -> Result<bool, DataStoreError> {
        let key = self.accessed_key(id);
        let exists: bool = self.con.clone().exists(key).await?;
        return Ok(exists);
    }

    #[instrument(skip(self), err)]
    async fn mark_as_accessed(&self, id: Uuid) -> Result<(), DataStoreError> {
        let key = self.accessed_key(id);
        let value = timestamp::now_string()?;

        let _: () = self
            .con
            .clone()
            .set_ex(key, value, self.max_ttl.as_secs())
            .await?;
        Ok(())
    }
}
