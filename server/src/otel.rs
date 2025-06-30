use std::time::Duration;

use anyhow::Result;
use log::info;

use opentelemetry::global;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_resource_detectors::{OsResourceDetector, ProcessResourceDetector};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    resource::{
        EnvResourceDetector, ResourceDetector, SdkProvidedResourceDetector,
        TelemetryResourceDetector,
    },
    runtime,
};

pub fn init_otel() -> Result<()> {
    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_err() {
        tracing_subscriber::fmt().init();
        info!("OpenTelemetry is not configured, skipping initialization.");
        return Ok(());
    }

    init_otel_logging()?;
    init_otel_tracing()?;
    init_otel_metrics()?;

    Ok(())
}

fn get_resource() -> Resource {
    let os_resource = OsResourceDetector.detect(Duration::from_secs(0));
    let process_resource = ProcessResourceDetector.detect(Duration::from_secs(0));
    let sdk_resource = SdkProvidedResourceDetector.detect(Duration::from_secs(0));
    let env_resource = EnvResourceDetector::new().detect(Duration::from_secs(0));
    let telemetry_resource = TelemetryResourceDetector.detect(Duration::from_secs(0));

    os_resource
        .merge(&process_resource)
        .merge(&sdk_resource)
        .merge(&env_resource)
        .merge(&telemetry_resource)
}

fn init_otel_tracing() -> Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default().with_resource(get_resource()),
        )
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(runtime::Tokio)?;

    global::set_tracer_provider(tracer_provider);

    Ok(())
}

fn init_otel_metrics() -> Result<()> {
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_resource(get_resource())
        .with_delta_temporality()
        .build()?;

    global::set_meter_provider(meter_provider);

    Ok(())
}

fn init_otel_logging() -> Result<()> {
    let logger_provider = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_resource(get_resource())
        .install_batch(runtime::Tokio)?;

    let stdout_logger = Box::new(
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Info)
            .build(),
    );

    let otel_log_appender = OpenTelemetryLogBridge::new(&logger_provider);
    let otel_logger = Box::new(otel_log_appender);

    multi_log::MultiLogger::init(vec![stdout_logger, otel_logger], log::Level::Info)?;

    Ok(())
}
