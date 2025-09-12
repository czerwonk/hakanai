// SPDX-License-Identifier: Apache-2.0

//! Stats observer for secret lifecycle events.
//!
//! This module provides an observer implementation that records statistics on a per-secret level.
//! The data can then be used for aggregated statistics and analysis.
//! No sensitive information is recorded.

use async_trait::async_trait;
use tracing::{error, instrument};
use uuid::Uuid;

use super::SecretStats;
use super::stats_store::StatsStore;
use crate::observer::{SecretEventContext, SecretObserver};

/// Observer that records per secret statistics.
pub struct StatsObserver {
    store: StatsStore,
}

impl StatsObserver {
    /// Create a new stats observer with a reference to the stats store.
    pub fn new(store: StatsStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl SecretObserver for StatsObserver {
    #[instrument(skip(self, context))]
    async fn on_secret_created(&self, secret_id: Uuid, context: &SecretEventContext) {
        let stat = SecretStats::new(context.ttl.unwrap_or_default().as_secs());
        let store = self.store.clone();
        tokio::spawn(async move {
            if let Err(e) = store.store_stats(secret_id, &stat).await {
                error!("Failed to store stats for secret {secret_id}: {e}");
            }
        });
    }

    #[instrument(skip(self, _context))]
    async fn on_secret_retrieved(&self, secret_id: Uuid, _context: &SecretEventContext) {
        let store = self.store.clone();
        tokio::spawn(async move {
            if let Err(e) = store.update_retrieved_at(secret_id).await {
                error!("Failed to update stats with retrieved_at for secret {secret_id}: {e}");
            }
        });
    }
}
