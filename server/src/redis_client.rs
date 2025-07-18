use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tracing::instrument;
use uuid::Uuid;

use hakanai_lib::timestamp;

use crate::data_store::{DataStore, DataStoreError, DataStorePopResult};

/// An implementation of the `DataStore` trait that uses Redis as its backend.
/// This struct holds a `ConnectionManager` for interacting with the Redis
/// server. It is designed to be cloneable and thread-safe.
#[derive(Clone)]
pub struct RedisClient {
    con: ConnectionManager,
    max_ttl: Duration,
}

impl RedisClient {
    pub async fn new(redis_url: &str, max_ttl: Duration) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let con = ConnectionManager::new(client).await?;
        Ok(Self { con, max_ttl })
    }
}

#[async_trait]
impl DataStore for RedisClient {
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

    #[instrument(skip(self), err)]
    async fn is_healthy(&self) -> Result<(), DataStoreError> {
        let _: () = self.con.clone().ping().await?;
        Ok(())
    }
}

impl RedisClient {
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
