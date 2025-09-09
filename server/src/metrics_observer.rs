// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry metrics observer for secret lifecycle events.
//!
//! This module provides an observer implementation that records metrics
//! for secret creation and retrieval events when OpenTelemetry is configured.

use async_trait::async_trait;
use opentelemetry::KeyValue;
use tracing::instrument;
use uuid::Uuid;

use crate::metrics::EventMetrics;
use crate::observer::{SecretEventContext, SecretObserver};

/// Observer that records OpenTelemetry metrics for secret events.
///
/// This observer is registered only when OpenTelemetry is configured
/// and records real-time metrics for secret lifecycle events.
pub struct MetricsObserver {
    /// Reference to the event metrics instruments
    metrics: EventMetrics,
}

impl MetricsObserver {
    /// Create a new metrics observer with a reference to the event metrics.
    pub fn new(metrics: EventMetrics) -> Self {
        Self { metrics }
    }
}

#[async_trait]
impl SecretObserver for MetricsObserver {
    #[instrument(skip(self, context))]
    async fn on_secret_created(&self, _secret_id: Uuid, context: &SecretEventContext) {
        // Determine user type for labeling
        let user_type = context
            .user_type
            .as_ref()
            .map(|ut| ut.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let labels = &[KeyValue::new("user_type", user_type)];

        // Record secret size distribution
        if let Some(size) = context.size {
            self.metrics
                .secret_size_histogram
                .record(size as u64, labels);
        }

        // Record TTL distribution
        if let Some(ttl) = context.ttl {
            self.metrics
                .secret_ttl_histogram
                .record(ttl.as_secs(), labels);
        }

        // Count total secrets created
        self.metrics.secrets_created_counter.add(1, labels);

        // Count secrets with restrictions
        if context.restrictions.is_some() {
            self.metrics
                .secrets_with_restrictions_counter
                .add(1, labels);
        }
    }

    #[instrument(skip(self, _context))]
    async fn on_secret_retrieved(&self, _secret_id: Uuid, _context: &SecretEventContext) {
        // No user_type available during retrieval - it's anonymous by design
        self.metrics.secrets_retrieved_counter.add(1, &[]);
    }
}
