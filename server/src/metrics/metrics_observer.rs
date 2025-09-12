// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry metrics observer for secret lifecycle events.
//!
//! This module provides an observer implementation that records metrics
//! for secret creation and retrieval events

use async_trait::async_trait;
use opentelemetry::KeyValue;
use tracing::instrument;
use uuid::Uuid;

use hakanai_lib::models::SecretRestrictions;

use super::event_metrics::EventMetrics;
use crate::observer::{SecretEventContext, SecretObserver};

/// Observer that records OpenTelemetry metrics for secret events.
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
        if let Some(ref restrictions) = context.restrictions {
            let bitfield = bitfield_value_for_restrictions(restrictions);
            let mut restriction_labels = labels.to_vec();
            restriction_labels.push(KeyValue::new("type", format!("{bitfield}")));

            self.metrics
                .secrets_with_restrictions_counter
                .add(1, &restriction_labels);
        }
    }

    #[instrument(skip(self, _context))]
    async fn on_secret_retrieved(&self, _secret_id: Uuid, _context: &SecretEventContext) {
        self.metrics.secrets_retrieved_counter.add(1, &[]);
    }
}

/// Compute a bitfield value representing the types of restrictions applied to a secret.
/// The bitfield uses the following mapping:
/// - Bit 0 (1): IP restrictions
/// - Bit 1 (2): Country restrictions
/// - Bit 2 (4): ASN restrictions
/// - Bit 3 (8): Passphrase protection
fn bitfield_value_for_restrictions(restrictions: &SecretRestrictions) -> u32 {
    let mut bitfield = 0;

    if restrictions.allowed_ips.is_some() {
        bitfield |= 1;
    }

    if restrictions.allowed_countries.is_some() {
        bitfield |= 1 << 1;
    }

    if restrictions.allowed_asns.is_some() {
        bitfield |= 1 << 2;
    }

    if restrictions.passphrase_hash.is_some() {
        bitfield |= 1 << 3;
    }

    bitfield
}

#[cfg(test)]
mod tests {
    use super::*;
    use hakanai_lib::models::{CountryCode, SecretRestrictions};

    #[test]
    fn test_bitfield_value_for_restrictions_ip_only() {
        let restrictions =
            SecretRestrictions::default().with_allowed_ips(vec!["127.0.0.1/32".parse().unwrap()]);
        let value = bitfield_value_for_restrictions(&restrictions);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_bitfield_value_for_restrictions_contry_only() {
        let restrictions = SecretRestrictions::default()
            .with_allowed_countries(vec![CountryCode::new("DE").unwrap()]);
        let value = bitfield_value_for_restrictions(&restrictions);
        assert_eq!(value, 2);
    }

    #[test]
    fn test_bitfield_value_for_restrictions_asn_only() {
        let restrictions = SecretRestrictions::default().with_allowed_asns(vec![202739]);
        let value = bitfield_value_for_restrictions(&restrictions);
        assert_eq!(value, 4);
    }

    #[test]
    fn test_bitfield_value_for_restrictions_passphrase_only() {
        let restrictions = SecretRestrictions::default().with_passphrase(b"test");
        let value = bitfield_value_for_restrictions(&restrictions);
        assert_eq!(value, 8);
    }

    #[test]
    fn test_bitfield_value_for_restrictions_all_set() {
        let restrictions = SecretRestrictions::default()
            .with_allowed_ips(vec!["127.0.0.1/32".parse().unwrap()])
            .with_allowed_asns(vec![202739])
            .with_allowed_countries(vec![CountryCode::new("DE").unwrap()])
            .with_passphrase(b"test");
        let value = bitfield_value_for_restrictions(&restrictions);
        assert_eq!(value, 15);
    }
}
