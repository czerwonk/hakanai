use anyhow::Result;

use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider};

use tracing_subscriber::{EnvFilter, prelude::*};

pub fn init() -> Result<()> {
    init_otel_logging()?;
    init_otel_tracing()?;

    Ok(())
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

fn init_otel_tracing() -> Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    if !is_otel_endpoint_set() {
        tracing::warn!(
            "OTEL_EXPORTER_OTLP_ENDPOINT is not set, OpenTelemetry tracing will not be exported."
        );
        return Ok(());
    }

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(provider.clone());

    Ok(())
}

fn init_otel_logging() -> Result<()> {
    let filter = EnvFilter::new("info");

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter);

    let builder = tracing_subscriber::registry().with(fmt_layer);

    if is_otel_endpoint_set() {
        let exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .build()?;
        let provider = SdkLoggerProvider::builder()
            .with_resource(get_resource())
            .with_batch_exporter(exporter)
            .build();
        let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(filter);

        builder.with(otel_layer).init();
    } else {
        builder.init();
    }

    Ok(())
}
