// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use super::secret_stats::SecretStats;

#[async_trait]
pub trait StatsStore: Send + Sync {
    /// Store the stats for the given secret ID.
    async fn store_stats(&self, secret_id: Uuid, stats: &SecretStats) -> Result<()>;

    /// Update the `retrieved_at` field of the stats for the given secret ID.
    async fn update_retrieved_at(&self, secret_id: Uuid) -> Result<Option<SecretStats>>;

    /// Retrieve all stored secret stats.
    async fn get_all_stats(&self) -> Result<Vec<SecretStats>>;
}
