use anyhow::Result;

use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::{
    metrics::SdkMeterProvider, propagation::TraceContextPropagator, trace::SdkTracerProvider,
};

use tracing::warn;
use tracing_subscriber::{EnvFilter, prelude::*};

/// A handler for OpenTelemetry providers.
///
/// This struct holds the tracer and meter providers. When `shutdown` is called,
/// the providers will be shut down gracefully.
pub struct Handler {
    tracing: SdkTracerProvider,
    metrics: SdkMeterProvider,
}

impl Handler {
    /// Shuts down the OpenTelemetry providers.
    ///
    /// This function should be called before the application exits to ensure
    /// that all telemetry data is exported.
    pub fn shutdown(&self) {
        if let Err(err) = self.tracing.shutdown() {
            warn!("Failed to shutdown tracing provider: {}", err);
        }
        if let Err(err) = self.metrics.shutdown() {
            warn!("Failed to shutdown metrics provider: {}", err);
        }
    }
}

/// Initializes OpenTelemetry tracing, metrics, and logging.
///
/// This function sets up the global tracer, meter, and logger providers.
/// It configures the OTLP exporter to send data to the endpoint specified
/// by the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable.
///
/// If the `OTEL_EXPORTER_OTLP_ENDPOINT` variable is not set, OpenTelemetry
/// will not be initialized, and this function will return `Ok(None)`.
///
/// # Returns
///
/// * `Ok(Some(OtelHandler))` - If OpenTelemetry was initialized successfully. The handler can be used to gracefully shut down the providers.
/// * `Ok(None)` - If the OTLP endpoint is not configured.
/// * `Err(anyhow::Error)` - If there was an error during initialization.
pub fn init() -> Result<Option<Handler>> {
    init_logging()?;

    if !is_otel_endpoint_set() {
        tracing::warn!(
            "OTEL_EXPORTER_OTLP_ENDPOINT is not set, OpenTelemetry traces and metrics will not be exported."
        );
        return Ok(None);
    }

    let tracer_provider = init_tracing()?;
    let meter_provider = init_metrics()?;

    Ok(Some(Handler {
        tracing: tracer_provider,
        metrics: meter_provider,
    }))
}

fn is_otel_endpoint_set() -> bool {
    std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok()
}

fn get_resource() -> Resource {
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "hakanai-server".to_string());
    Resource::builder()
        .with_service_name(service_name)
        .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
        .build()
}

fn init_logging() -> Result<()> {
    let fmt_filter = EnvFilter::new("info");
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(fmt_filter);

    let builder = tracing_subscriber::registry().with(fmt_layer);

    if is_otel_endpoint_set() {
        let exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .build()?;
        let provider = SdkLoggerProvider::builder()
            .with_resource(get_resource())
            .with_batch_exporter(exporter)
            .build();
        let otel_filter = EnvFilter::new("info");
        let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(otel_filter);

        builder.with(otel_layer).init();
    } else {
        builder.init();
    }

    Ok(())
}

fn init_tracing() -> Result<SdkTracerProvider> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

fn init_metrics() -> Result<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .build()?;
    let provider = SdkMeterProvider::builder()
        .with_resource(get_resource())
        .with_periodic_exporter(exporter)
        .build();

    global::set_meter_provider(provider.clone());

    Ok(provider)
}
