// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use opentelemetry::global;
use opentelemetry::metrics::Gauge;
use tokio::time::interval;
use tracing::error;

use crate::stats::StatsStore;
use crate::token::TokenStore;

/// Metrics collector for the Hakanai server.
///
/// This struct manages OpenTelemetry metrics collection and periodic updates.
pub struct MetricsCollector {
    /// Gauge for tracking active token count
    token_count_gauge: Gauge<u64>,

    /// Gauge for tracking active secret count
    active_secret_count_gauge: Gauge<u64>,

    /// Gauge for tracking expired secret count
    expired_secret_count_gauge: Gauge<u64>,
}

impl MetricsCollector {
    /// Create a new metrics collector using the global meter provider.
    pub fn new() -> Self {
        let meter = global::meter(super::METER_NAME);
        let token_count_gauge = meter
            .u64_gauge("hakanai_active_tokens")
            .with_description("Number of active user tokens")
            .build();
        let active_secret_count_gauge = meter
            .u64_gauge("hakanai_active_secrets")
            .with_description("Number of active secrets stored")
            .build();
        let expired_secret_count_gauge = meter
            .u64_gauge("hakanai_expired_secrets_total")
            .with_description("Number of expired secrets")
            .build();

        Self {
            token_count_gauge,
            active_secret_count_gauge,
            expired_secret_count_gauge,
        }
    }

    /// Start periodic metrics collection in the background.
    ///
    /// This method spawns a background task that periodically collects
    /// metrics from the token store and data store and updates the OpenTelemetry gauges.
    pub fn start_collection<T: TokenStore + 'static, S: StatsStore + 'static>(
        &self,
        token_store: Arc<T>,
        stats_store: Arc<S>,
        interval_duration: Duration,
    ) {
        let token_count_gauge = self.token_count_gauge.clone();
        let secret_count_gauge = self.active_secret_count_gauge.clone();
        let expired_secret_count_gauge = self.expired_secret_count_gauge.clone();

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                collect_token_metrics(&token_store, &token_count_gauge).await;

                if let Err(err) = collect_secret_metrics(
                    &stats_store,
                    &secret_count_gauge,
                    &expired_secret_count_gauge,
                )
                .await
                {
                    error!("Failed to collect secret metrics: {}", err);
                }
            }
        });
    }
}

async fn collect_token_metrics<T: TokenStore>(
    token_store: &Arc<T>,
    token_count_gauge: &Gauge<u64>,
) {
    match token_store.user_token_count().await {
        Ok(count) => {
            token_count_gauge.record(count as u64, &[]);
        }
        Err(err) => {
            error!("Failed to get token count for metrics: {}", err);
        }
    }
}

async fn collect_secret_metrics<S: StatsStore>(
    stats_store: &Arc<S>,
    active_secret_count_gauge: &Gauge<u64>,
    expired_secret_count_gauge: &Gauge<u64>,
) -> Result<()> {
    let stats = stats_store.get_all_stats().await?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut active: u64 = 0;
    let mut expired: u64 = 0;

    for stat in stats {
        if stat.has_expired(timestamp) {
            expired += 1;
        } else if stat.retrieved_at.is_none() {
            active += 1;
        }
    }

    expired_secret_count_gauge.record(expired, &[]);
    active_secret_count_gauge.record(active, &[]);
    Ok(())
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
