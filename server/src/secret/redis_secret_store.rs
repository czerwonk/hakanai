// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tracing::instrument;
use uuid::Uuid;

use hakanai_lib::models::SecretRestrictions;
use hakanai_lib::utils::timestamp;

use crate::secret::{SecretStore, SecretStoreError, SecretStorePopResult};

const SECRET_PREFIX: &str = "secret:";
const ACCESSED_PREFIX: &str = "accessed:";
const RESTRICTIONS_PREFIX: &str = "restrictions:";

/// An implementation of the `SecretStore` trait that uses Redis as its backend.
/// This struct holds a `ConnectionManager` for interacting with the Redis
/// server. It is designed to be cloneable and thread-safe.
#[derive(Clone)]
pub struct RedisSecretStore {
    con: ConnectionManager,
    max_ttl: Duration,
}

impl RedisSecretStore {
    pub fn new(con: ConnectionManager, max_ttl: Duration) -> Self {
        Self { con, max_ttl }
    }
}

impl RedisSecretStore {
    fn secret_key(&self, id: Uuid) -> String {
        format!("{SECRET_PREFIX}{id}")
    }

    fn accessed_key(&self, id: Uuid) -> String {
        format!("{ACCESSED_PREFIX}{id}")
    }

    fn restrictions_key(&self, id: Uuid) -> String {
        format!("{RESTRICTIONS_PREFIX}{id}")
    }

    #[instrument(skip(self), err)]
    async fn was_accessed(&self, id: Uuid) -> Result<bool, SecretStoreError> {
        let key = self.accessed_key(id);
        let exists: bool = self.con.clone().exists(key).await?;
        return Ok(exists);
    }

    #[instrument(skip(self), err)]
    async fn mark_as_accessed(&self, id: Uuid) -> Result<(), SecretStoreError> {
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
impl SecretStore for RedisSecretStore {
    #[instrument(skip(self), err)]
    async fn pop(&self, id: Uuid) -> Result<SecretStorePopResult, SecretStoreError> {
        let secret_key = self.secret_key(id);
        let value = self.con.clone().get_del(secret_key).await?;

        if let Some(secret) = value {
            self.mark_as_accessed(id).await?;
            return Ok(SecretStorePopResult::Found(secret));
        }

        if self.was_accessed(id).await? {
            return Ok(SecretStorePopResult::AlreadyAccessed);
        }

        Ok(SecretStorePopResult::NotFound)
    }

    #[instrument(skip(self, data), err)]
    async fn put(
        &self,
        id: Uuid,
        data: String,
        expires_in: Duration,
    ) -> Result<(), SecretStoreError> {
        let secret_key = self.secret_key(id);
        let _: () = self
            .con
            .clone()
            .set_ex(secret_key, data, expires_in.as_secs())
            .await?;
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn is_healthy(&self) -> Result<(), SecretStoreError> {
        let _: () = self.con.clone().ping().await?;
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn active_secret_count(&self) -> Result<usize, SecretStoreError> {
        let pattern = format!("{SECRET_PREFIX}*");
        let keys: Vec<String> = self.con.clone().keys(pattern).await?;
        Ok(keys.len())
    }

    #[instrument(skip(self), err)]
    async fn set_restrictions(
        &self,
        id: Uuid,
        restrictions: &SecretRestrictions,
        expires_in: Duration,
    ) -> Result<(), SecretStoreError> {
        let key = self.restrictions_key(id);
        let json = serde_json::to_string(restrictions)?;

        let _: () = self
            .con
            .clone()
            .set_ex(key, json, expires_in.as_secs())
            .await?;
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_restrictions(
        &self,
        id: Uuid,
    ) -> Result<Option<SecretRestrictions>, SecretStoreError> {
        let key = self.restrictions_key(id);
        let value: Option<String> = self.con.clone().get(key).await?;

        match value {
            Some(json) => {
                let restrictions = serde_json::from_str(&json)?;
                Ok(Some(restrictions))
            }
            None => Ok(None),
        }
    }
}
