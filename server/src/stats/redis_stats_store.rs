// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use anyhow::Result;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use uuid::Uuid;

use super::SecretStats;

/// Stores and retrieves secret statistics using Redis.
#[derive(Clone)]
pub struct RedisStatsStore {
    con: ConnectionManager,
    ttl: Duration,
}

impl RedisStatsStore {
    /// Create a new stats observer with a Redis client.
    pub fn new(con: ConnectionManager, ttl: Duration) -> Self {
        Self { con, ttl }
    }

    fn key(secret_id: Uuid) -> String {
        format!("stats:{}", secret_id)
    }

    /// Store the stats for the given secret ID.
    pub async fn store_stats(&self, secret_id: Uuid, stats: &SecretStats) -> Result<()> {
        let key = Self::key(secret_id);
        let value = serde_json::to_string(stats)?;

        let _: () = self
            .con
            .clone()
            .set_ex(key, value, self.ttl.as_secs())
            .await?;

        Ok(())
    }

    /// Update the `retrieved_at` field of the stats for the given secret ID.
    pub async fn update_retrieved_at(&self, secret_id: Uuid) -> Result<()> {
        if let Some(mut stat) = self.retrieve_stats(secret_id).await? {
            let retrieved_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            stat.retrieved_at = Some(retrieved_at);
            self.store_stats(secret_id, &stat).await?;
        }

        Ok(())
    }

    async fn retrieve_stats(&self, secret_id: Uuid) -> Result<Option<SecretStats>> {
        let key = Self::key(secret_id);
        let value: Option<String> = self.con.clone().get(key).await?;

        if let Some(json) = value {
            let stats = serde_json::from_str(&json)?;
            return Ok(Some(stats));
        }

        Ok(None)
    }
}
