use anyhow::{Context, Result};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use prometheus::Registry;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::ObservabilityConfig;

pub struct Observability {
    pub prometheus_registry: Registry,
    tracer_provider: Option<SdkTracerProvider>,
}

pub fn init(config: &ObservabilityConfig) -> Result<Observability> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("adeptus={level}", level = config.logging.level))
    });

    let prometheus_registry = Registry::new();
    let prometheus_exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus_registry.clone())
        .build()
        .context("Failed to build Prometheus exporter")?;

    let _meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(prometheus_exporter)
        .build();

    let tracer_provider = if config.tracing.enabled {
        let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.tracing.otlp_endpoint)
            .build()
            .context("Failed to build OTLP exporter")?;

        let provider = SdkTracerProvider::builder()
            .with_sampler(Sampler::TraceIdRatioBased(config.tracing.sampling_ratio))
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_attributes([
                        KeyValue::new("service.name", "adeptus"),
                        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ])
                    .build(),
            )
            .with_batch_exporter(otlp_exporter)
            .build();

        opentelemetry::global::set_tracer_provider(provider.clone());

        let otel_layer = tracing_opentelemetry::layer();

        match config.logging.format.as_str() {
            "json" => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(otel_layer)
                    .with(tracing_subscriber::fmt::layer().json())
                    .init();
            }
            _ => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(otel_layer)
                    .with(tracing_subscriber::fmt::layer())
                    .init();
            }
        }

        info!(
            "OpenTelemetry tracing enabled (endpoint: {})",
            config.tracing.otlp_endpoint
        );

        Some(provider)
    } else {
        match config.logging.format.as_str() {
            "json" => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer().json())
                    .init();
            }
            _ => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer())
                    .init();
            }
        }

        None
    };

    if config.metrics.enabled {
        info!("Prometheus metrics enabled at /metrics");
    }

    Ok(Observability {
        prometheus_registry,
        tracer_provider,
    })
}

pub fn shutdown(obs: &Observability) {
    if let Some(provider) = &obs.tracer_provider
        && let Err(e) = provider.shutdown()
    {
        tracing::error!("Failed to shutdown tracer provider: {e}");
    }
}
