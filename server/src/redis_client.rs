use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tracing::instrument;
use uuid::Uuid;

use hakanai_lib::timestamp;

use crate::data_store::{DataStore, DataStoreError, DataStorePopResult};
use crate::token::{TokenData, TokenError, TokenStore};

const ADMIN_TOKEN_KEY: &str = "admin_token";

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

impl RedisClient {
    fn accessed_key(&self, id: Uuid) -> String {
        format!("accessed:{id}")
    }

    fn token_key(&self, hash: &str) -> String {
        format!("token:{hash}")
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

#[async_trait]
impl TokenStore for RedisClient {
    #[instrument(skip(self), err)]
    async fn is_empty(&self) -> Result<bool, TokenError> {
        let keys: Vec<String> = self.con.clone().keys("token:*").await?;
        Ok(keys.is_empty())
    }

    #[instrument(skip(self), err)]
    async fn get_token(&self, token_hash: &str) -> Result<Option<TokenData>, TokenError> {
        let key = self.token_key(token_hash);
        let value: Option<String> = self.con.clone().get(key).await?;

        if let Some(data) = value {
            let token_data = TokenData::deserialize(&data)?;
            return Ok(Some(token_data));
        }

        Ok(None)
    }

    #[instrument(skip(self), err)]
    async fn store_token(
        &self,
        token_hash: &str,
        ttl: Duration,
        token_data: TokenData,
    ) -> Result<(), TokenError> {
        let data = token_data.serialize()?;
        let key = self.token_key(token_hash);
        let _: () = self.con.clone().set_ex(key, data, ttl.as_secs()).await?;
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn clear_all_user_tokens(&self) -> Result<(), TokenError> {
        let keys: Vec<String> = self.con.clone().keys("token:*").await?;
        if !keys.is_empty() {
            let _: () = self.con.clone().del(keys).await?;
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn admin_token_exists(&self) -> Result<bool, TokenError> {
        let exists: bool = self.con.clone().exists(ADMIN_TOKEN_KEY).await?;
        Ok(exists)
    }

    #[instrument(skip(self), err)]
    async fn get_admin_token(&self) -> Result<Option<String>, TokenError> {
        let value: Option<String> = self.con.clone().get(ADMIN_TOKEN_KEY).await?;
        Ok(value)
    }

    #[instrument(skip(self), err)]
    async fn store_admin_token(&self, token_hash: &str) -> Result<(), TokenError> {
        let _: () = self.con.clone().set(ADMIN_TOKEN_KEY, token_hash).await?;
        Ok(())
    }
}
