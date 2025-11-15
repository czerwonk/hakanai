// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tracing::instrument;

use super::{TokenData, TokenError, TokenStore};

const ADMIN_TOKEN_KEY: &str = "admin_token";
const TOKEN_PREFIX: &str = "token:";

/// An implementation of the `TokenStore` trait that uses Redis as its backend.
#[derive(Clone)]
pub struct RedisTokenStore {
    con: ConnectionManager,
}

impl RedisTokenStore {
    pub fn new(con: ConnectionManager) -> Self {
        Self { con }
    }
}

impl RedisTokenStore {
    fn token_key(&self, hash: &str) -> String {
        format!("{TOKEN_PREFIX}{hash}")
    }

    async fn delete_if_one_time(
        &self,
        key: &str,
        token_data: &TokenData,
    ) -> Result<(), TokenError> {
        if !token_data.one_time {
            return Ok(());
        }

        let _: () = self.con.clone().del(key).await?;
        Ok(())
    }
}

#[async_trait]
impl TokenStore for RedisTokenStore {
    #[instrument(skip(self), err)]
    async fn get_token(&self, token_hash: &str) -> Result<Option<TokenData>, TokenError> {
        let key = self.token_key(token_hash);
        let value: Option<String> = self.con.clone().get(&key).await?;

        if let Some(data) = value {
            let token_data = TokenData::deserialize(&data)?;
            self.delete_if_one_time(&key, &token_data).await?;
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
        let keys: Vec<String> = self.con.clone().keys(format!("{TOKEN_PREFIX}*")).await?;
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

    #[instrument(skip(self), err)]
    async fn user_token_count(&self) -> Result<usize, TokenError> {
        let keys: Vec<String> = self.con.clone().keys(format!("{TOKEN_PREFIX}*")).await?;
        Ok(keys.len())
    }
}
