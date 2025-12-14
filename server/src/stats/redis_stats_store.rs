// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tracing::warn;
use uuid::Uuid;

use crate::stats::StatsStore;

use super::secret_stats::SecretStats;

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

    /// Retrieve the stats for the given secret ID.
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

#[async_trait]
impl StatsStore for RedisStatsStore {
    /// Store the stats for the given secret ID.
    async fn store_stats(&self, secret_id: Uuid, stats: &SecretStats) -> Result<()> {
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
    async fn update_retrieved_at(&self, secret_id: Uuid) -> Result<Option<SecretStats>> {
        if let Some(mut stat) = self.retrieve_stats(secret_id).await? {
            let retrieved_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            stat.retrieved_at = Some(retrieved_at);
            self.store_stats(secret_id, &stat).await?;
            return Ok(Some(stat));
        }

        Ok(None)
    }

    /// Retrieve all stats stored in Redis using SCAN for better performance.
    async fn get_all_stats(&self) -> Result<Vec<SecretStats>> {
        let mut stats = Vec::new();
        let mut con = self.con.clone();
        let mut cursor = 0u64;
        let mut i = 0;

        const KEYS_PER_SCAN: usize = 100;
        const MAX_ITERATIONS: usize = 10_000;

        loop {
            i += 1;
            if i > MAX_ITERATIONS {
                warn!("Max iterations reached while scanning Redis stats keys");
                break; // Prevent infinite loops in case of unexpected behavior
            }

            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .cursor_arg(cursor)
                .arg("MATCH")
                .arg("stats:*")
                .arg("COUNT")
                .arg(KEYS_PER_SCAN)
                .query_async(&mut con)
                .await?;

            // Fetch values for all keys in this batch
            if !keys.is_empty() {
                let values: Vec<Option<String>> = con.mget(keys).await?;
                for value in values.into_iter().flatten() {
                    if let Ok(stat) = serde_json::from_str(&value) {
                        stats.push(stat);
                    }
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break; // Scan complete when cursor returns to 0
            }
        }

        Ok(stats)
    }
}
