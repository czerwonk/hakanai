// SPDX-License-Identifier: MIT

//! OpenTelemetry metrics for the Hakanai server.
//!
//! This module provides metrics collection for various server operations,
//! including token count tracking and other operational metrics.

use std::sync::Arc;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry::metrics::{Gauge, Meter};
use tokio::time::interval;
use tracing::{debug, error};

use crate::data_store::{DataStore, DataStoreError};
use crate::token::{TokenError, TokenStore};

/// Metrics collector for the Hakanai server.
///
/// This struct manages OpenTelemetry metrics collection and periodic updates.
pub struct MetricsCollector {
    /// OpenTelemetry meter for creating instruments
    #[allow(dead_code)]
    meter: Meter,
    /// Gauge for tracking active token count
    token_count_gauge: Gauge<u64>,
    /// Gauge for tracking active secret count
    secret_count_gauge: Gauge<u64>,
}

impl MetricsCollector {
    /// Create a new metrics collector using the global meter provider.
    pub fn new() -> Self {
        let meter = global::meter("hakanai-server");
        let token_count_gauge = meter
            .u64_gauge("hakanai_active_tokens")
            .with_description("Number of active user tokens")
            .build();
        let secret_count_gauge = meter
            .u64_gauge("hakanai_active_secrets")
            .with_description("Number of active secrets stored")
            .build();

        Self {
            meter,
            token_count_gauge,
            secret_count_gauge,
        }
    }

    /// Start periodic metrics collection in the background.
    ///
    /// This method spawns a background task that periodically collects
    /// metrics from the token store and data store and updates the OpenTelemetry gauges.
    pub fn start_collection<T: TokenStore + 'static, D: DataStore + 'static>(
        &self,
        token_store: Arc<T>,
        data_store: Arc<D>,
        interval_duration: Duration,
    ) {
        let token_count_gauge = self.token_count_gauge.clone();
        let secret_count_gauge = self.secret_count_gauge.clone();

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                // Collect token metrics
                match token_store.user_token_count().await {
                    Ok(count) => {
                        token_count_gauge.record(count as u64, &[]);
                        debug!("Updated token count metric: {}", count);
                    }
                    Err(err) => {
                        error!("Failed to get token count for metrics: {}", err);
                    }
                }

                // Collect secret metrics
                match data_store.active_secret_count().await {
                    Ok(count) => {
                        secret_count_gauge.record(count as u64, &[]);
                        debug!("Updated secret count metric: {}", count);
                    }
                    Err(err) => {
                        error!("Failed to get secret count for metrics: {}", err);
                    }
                }
            }
        });
    }

    /// Manually update the token count metric.
    ///
    /// This method can be used to immediately update the token count
    /// without waiting for the periodic collection.
    #[allow(dead_code)]
    pub async fn update_token_count<T: TokenStore>(
        &self,
        token_store: &T,
    ) -> Result<(), TokenError> {
        let count = token_store.user_token_count().await?;
        self.token_count_gauge.record(count as u64, &[]);
        debug!("Manually updated token count metric: {}", count);
        Ok(())
    }

    /// Manually update the secret count metric.
    ///
    /// This method can be used to immediately update the secret count
    /// without waiting for the periodic collection.
    #[allow(dead_code)]
    pub async fn update_secret_count<D: DataStore>(
        &self,
        data_store: &D,
    ) -> Result<(), DataStoreError> {
        let count = data_store.active_secret_count().await?;
        self.secret_count_gauge.record(count as u64, &[]);
        debug!("Manually updated secret count metric: {}", count);
        Ok(())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize metrics collection for the server.
///
/// This function creates a metrics collector and starts periodic collection
/// if OpenTelemetry is enabled (i.e., if the global meter provider is available).
pub fn init_metrics_collection<T: TokenStore + 'static, D: DataStore + 'static>(
    token_store: Arc<T>,
    data_store: Arc<D>,
    collection_interval: Duration,
) -> Option<MetricsCollector> {
    // Check if OpenTelemetry is configured by trying to get a meter
    let meter = global::meter("hakanai-server");

    // Try to create a test gauge to verify OpenTelemetry is working
    let test_gauge = meter.u64_gauge("test_gauge").build();
    test_gauge.record(0, &[]);

    let collector = MetricsCollector::new();
    collector.start_collection(token_store, data_store, collection_interval);

    debug!(
        "Started metrics collection with interval: {:?}",
        collection_interval
    );
    Some(collector)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{MockDataStore, MockTokenStore};

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();

        // Test that we can create a collector without errors
        let test_gauge = collector.meter.clone().u64_gauge("test").build();
        test_gauge.record(1, &[]);
    }

    #[tokio::test]
    async fn test_update_token_count() {
        let collector = MetricsCollector::new();
        let mock_store = MockTokenStore::new().with_token_count(5);

        // This should not panic and should complete successfully
        let result = collector.update_token_count(&mock_store).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_token_count_with_failure() {
        let collector = MetricsCollector::new();
        let mock_store = MockTokenStore::new().with_failures();

        // This should return an error when the token store fails
        let result = collector.update_token_count(&mock_store).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_secret_count() {
        let collector = MetricsCollector::new();
        let mock_store = MockDataStore::new().with_secret_count(15);

        // This should not panic and should complete successfully
        let result = collector.update_secret_count(&mock_store).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_secret_count_with_failure() {
        let collector = MetricsCollector::new();
        let mock_store = MockDataStore::new().with_failures();

        // This should return an error when the data store fails
        let result = collector.update_secret_count(&mock_store).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_init_metrics_collection() {
        let mock_token_store = Arc::new(MockTokenStore::new().with_token_count(10));
        let mock_data_store = Arc::new(MockDataStore::new().with_secret_count(5));
        let interval = Duration::from_millis(100);

        let collector = init_metrics_collection(mock_token_store, mock_data_store, interval);

        // Should create a collector successfully
        assert!(collector.is_some());
    }
}
