use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

/// The tracer provider is built once at startup and kept alive for the process's lifetime by
/// returning it to `main` for storage there — a provider dropped early silently stops exporting
/// spans (`assets-web`'s `application/tracing.py` has a comment about the exact same failure
/// mode hitting catalog-api via a misplaced `defer`). Returns `None` when
/// `OTEL_EXPORTER_OTLP_ENDPOINT` is unset, so tracing is a no-op rather than a startup failure in
/// local dev.
pub fn init(config: &Config) -> Option<opentelemetry_sdk::trace::SdkTracerProvider> {
    let env_filter =
        EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    let provider = config.otlp_endpoint.as_ref().map(|endpoint| {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(format!("{endpoint}/v1/traces"))
            .build()
            .expect("failed to build OTLP span exporter");

        opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .build()
    });

    let otel_layer = provider.as_ref().map(|p| {
        let tracer = p.tracer("sweetrpg-main-web");
        tracing_opentelemetry::layer().with_tracer(tracer)
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .with(otel_layer)
        .init();

    provider
}
