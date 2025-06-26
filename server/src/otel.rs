use actix_web::{App, HttpServer, middleware::Logger, web};
use opentelemetry::global;
use opentelemetry_otlp::{LogExporter, MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    logs::LoggerProvider,
    metrics::{MeterProvider, PeriodicReader},
    runtime::Tokio,
};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
        opentelemetry::KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
    ]);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default().with_resource(resource.clone()),
        )
        .install_batch(Tokio)?;

    let metric_reader =
        PeriodicReader::builder(MetricExporter::builder().with_tonic().build()?, Tokio).build();

    let meter_provider = MeterProvider::builder()
        .with_reader(metric_reader)
        .with_resource(resource.clone())
        .build();

    global::set_meter_provider(meter_provider);

    let logger_provider = LoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(LogExporter::builder().with_tonic().build()?, Tokio)
        .build();

    Ok(())
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
    global::shutdown_meter_provider();
}
