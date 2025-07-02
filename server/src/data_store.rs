use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use thiserror::Error;
use tracing::{error, instrument};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DataStoreError {
    #[error("web request failed")]
    Redis(#[from] redis::RedisError),

    #[error("internal error: {0}")]
    #[allow(dead_code)]
    InternalError(String),
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
    /// A `Result` which is `Ok(Some(String))` if the item is found, `Ok(None)`
    /// if the item is not found, or an `Err` if an error occurs during the
    /// operation.
    async fn pop(&self, id: Uuid) -> Result<Option<String>, DataStoreError>;

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
}

impl RedisDataStore {
    pub async fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let con = ConnectionManager::new(client).await?;
        Ok(Self { con })
    }
}

#[async_trait]
impl DataStore for RedisDataStore {
    #[instrument(skip(self), err)]
    async fn pop(&self, id: Uuid) -> Result<Option<String>, DataStoreError> {
        let value = self.con.clone().get_del(id.to_string()).await?;
        Ok(value)
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
