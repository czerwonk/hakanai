// SPDX-License-Identifier: Apache-2.0

use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram};

// Histogram bucket definitions
const SECRET_SIZE_BUCKETS: &[f64] = &[
    256.0,      // 256B
    1024.0,     // 1KB
    4096.0,     // 4KB
    16384.0,    // 16KB
    32768.0,    // 32KB (anonymous limit)
    65536.0,    // 64KB
    262144.0,   // 256KB
    1048576.0,  // 1MB
    4194304.0,  // 4MB
    10485760.0, // 10MB (default max)
];

const SECRET_TTL_BUCKETS: &[f64] = &[
    300.0,     // 5 minutes
    900.0,     // 15 minutes
    3600.0,    // 1 hour
    21600.0,   // 6 hours
    43200.0,   // 12 hours
    86400.0,   // 1 day
    259200.0,  // 3 days
    604800.0,  // 1 week
    1209600.0, // 2 weeks
    2592000.0, // 30 days
];

const SECRET_LIFETIME_BUCKETS: &[f64] = &[
    10.0,     // 10 seconds
    30.0,     // 30 seconds
    60.0,     // 1 minute
    300.0,    // 5 minutes
    900.0,    // 15 minutes
    3600.0,   // 1 hour
    10800.0,  // 3 hours
    43200.0,  // 12 hours
    86400.0,  // 1 day
    259200.0, // 3 days
    604800.0, // 1 week
];

/// Event-driven metrics for secret lifecycle events.
///
/// This struct contains counters and histograms that are updated
/// in real-time as secrets are created and retrieved.
#[derive(Clone)]
pub struct EventMetrics {
    /// Histogram for tracking secret sizes in bytes
    pub secret_size_histogram: Histogram<u64>,

    /// Histogram for tracking TTL values in seconds
    pub secret_ttl_histogram: Histogram<u64>,

    /// Counter for total secrets created
    pub secrets_created_counter: Counter<u64>,

    /// Counter for total secrets retrieved
    pub secrets_retrieved_counter: Counter<u64>,

    /// Counter for secrets created with restrictions
    pub secrets_with_restrictions_counter: Counter<u64>,

    /// Histogram for tracking secret lifetime from creation to retrieval
    pub secret_lifetime_histogram: Histogram<u64>,
}

impl EventMetrics {
    /// Create a new set of event metrics using the global meter provider.
    pub fn new() -> Self {
        let meter = global::meter(super::METER_NAME);

        Self {
            secret_size_histogram: meter
                .u64_histogram("hakanai_secret_size_bytes")
                .with_description("Distribution of secret sizes in bytes")
                .with_boundaries(SECRET_SIZE_BUCKETS.to_vec())
                .build(),

            secret_ttl_histogram: meter
                .u64_histogram("hakanai_secret_ttl_seconds")
                .with_description("Distribution of TTL values in seconds")
                .with_boundaries(SECRET_TTL_BUCKETS.to_vec())
                .build(),

            secrets_created_counter: meter
                .u64_counter("hakanai_secrets_created_total")
                .with_description("Total number of secrets created")
                .build(),

            secrets_retrieved_counter: meter
                .u64_counter("hakanai_secrets_retrieved_total")
                .with_description("Total number of secrets retrieved")
                .build(),

            secrets_with_restrictions_counter: meter
                .u64_counter("hakanai_secrets_with_restrictions_total")
                .with_description("Total number of secrets created with access restrictions")
                .build(),

            secret_lifetime_histogram: meter
                .u64_histogram("hakanai_secret_lifetime_seconds")
                .with_description("Time from secret creation to retrieval in seconds")
                .with_boundaries(SECRET_LIFETIME_BUCKETS.to_vec())
                .build(),
        }
    }
}
