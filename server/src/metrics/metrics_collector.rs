// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::Duration;

use opentelemetry::global;
use opentelemetry::metrics::{Gauge, Meter};
use tokio::time::interval;
use tracing::{debug, error};

use crate::secret::{SecretStore, SecretStoreError};
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
        let meter = global::meter(super::METER_NAME);
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
    pub fn start_collection<T: TokenStore + 'static, D: SecretStore + 'static>(
        &self,
        token_store: Arc<T>,
        secret_store: Arc<D>,
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
                match secret_store.active_secret_count().await {
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
    pub async fn update_secret_count<D: SecretStore>(
        &self,
        secret_store: &D,
    ) -> Result<(), SecretStoreError> {
        let count = secret_store.active_secret_count().await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret::MockSecretStore;
    use crate::token::MockTokenStore;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();

        // Test that we can create a collector without errors
        let test_gauge = collector.meter.clone().u64_gauge("test").build();
        test_gauge.record(1, &[]);
    }

    #[tokio::test]
    async fn test_update_token_count() -> Result<(), Box<dyn std::error::Error>> {
        let collector = MetricsCollector::new();
        let mock_store = MockTokenStore::new().with_token_count(5);

        // This should not panic and should complete successfully
        collector.update_token_count(&mock_store).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_token_count_with_failure() {
        let collector = MetricsCollector::new();
        let mock_store = MockTokenStore::new().with_failures();

        // This should return an error when the token store fails
        let result = collector.update_token_count(&mock_store).await;
        assert!(
            result.is_err(),
            "Expected error for token store failure, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_update_secret_count() -> Result<(), Box<dyn std::error::Error>> {
        let collector = MetricsCollector::new();
        let mock_store = MockSecretStore::new().with_secret_count(15);

        // This should not panic and should complete successfully
        collector.update_secret_count(&mock_store).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_secret_count_with_failure() {
        let collector = MetricsCollector::new();
        let mock_store = MockSecretStore::new().with_failures();

        // This should return an error when the data store fails
        let result = collector.update_secret_count(&mock_store).await;
        assert!(
            result.is_err(),
            "Expected error for data store failure, got: {:?}",
            result
        );
    }
}
