// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry metrics for the Hakanai server.
//!
//! This module provides metrics collection for various server operations,
//! including token count tracking and other operational metrics.

pub const METER_NAME: &str = "hakanai-server";

mod event_metrics;
mod metrics_collector;
mod metrics_observer;

pub use event_metrics::EventMetrics;
pub use metrics_collector::MetricsCollector;
pub use metrics_observer::MetricsObserver;
