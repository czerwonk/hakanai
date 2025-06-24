use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DataStoreError {
    #[error("web request failed")]
    Redis(#[from] redis::RedisError),

    #[error("internal error: {0}")]
    InternalError(String),
}

/// `DataStore` is a trait that defines the contract for a simple, asynchronous,
/// key-value storage system. Implementations of this trait are expected to be
/// thread-safe.
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Retrieves a value from the data store based on its `Uuid`.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the item to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Some(String))` if the item is found, `Ok(None)`
    /// if the item is not found, or an `Err` if an error occurs during the
    /// operation.
    async fn get(&mut self, id: Uuid) -> Result<Option<String>, DataStoreError>;

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
        &mut self,
        id: Uuid,
        data: String,
        expires_in: Duration,
    ) -> Result<(), DataStoreError>;
}

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
    async fn get(&mut self, id: Uuid) -> Result<Option<String>, DataStoreError> {
        let value = self.con.get(id.to_string()).await?;
        Ok(value)
    }

    async fn put(
        &mut self,
        id: Uuid,
        data: String,
        expires_in: Duration,
    ) -> Result<(), DataStoreError> {
        let _: () = self
            .con
            .set_ex(&id.to_string(), &data, expires_in.as_secs() as u64)
            .await?;
        Ok(())
    }
}
