use anyhow::Result;
use axum::{
    Router,
    http::StatusCode,
    middleware as axum_middleware,
    routing::{get, post},
};
use config::{Config, Environment, File};
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{error, info};

use adeptus::AppState;
use adeptus::config::AppConfig;
use adeptus::db::{DatabaseManager, RepositoryManager};
use adeptus::events::EventPublisher;
use adeptus::graphql::{build_schema, graphql_handler, graphql_playground};
use adeptus::handlers::{file, health, pdf};
use adeptus::keto::KetoClient;
use adeptus::middleware::{auth, rate_limit};
use adeptus::observability;
use adeptus::pactum_client::PactumClient;

fn load_config() -> Result<AppConfig> {
    let config = Config::builder()
        .add_source(File::with_name("config/default").required(false))
        .add_source(File::with_name("config/local").required(false))
        .add_source(Environment::with_prefix("ADEPTUS").separator("__"))
        .set_default("server.host", "0.0.0.0")?
        .set_default("server.port", 3000)?
        .set_default("server.timeout_seconds", 30)?
        .set_default("database.max_connections", 20)?
        .set_default("database.min_connections", 5)?
        .set_default("database.run_migrations", true)?
        .set_default("nats.stream_name", "ADEPTUS_EVENTS")?
        .set_default("nats.max_age_days", 30_i64)?
        .set_default("rate_limiting.enabled", true)?
        .set_default("rate_limiting.global_limit", 10000)?
        .set_default("rate_limiting.per_user_limit", 1000)?
        .set_default("rate_limiting.per_ip_limit", 100)?
        .set_default("rate_limiting.burst_size", 10)?
        .set_default("observability.logging.level", "info")?
        .set_default("observability.logging.format", "pretty")?
        .set_default("observability.tracing.enabled", false)?
        .set_default(
            "observability.tracing.otlp_endpoint",
            "http://localhost:4317",
        )?
        .set_default("observability.tracing.sampling_ratio", 0.1)?
        .set_default("observability.metrics.enabled", true)?
        .set_default("pdf.wkhtmltopdf_path", "wkhtmltopdf")?
        .set_default("pdf.temp_dir", "/tmp/adeptus-pdf")?
        .set_default("pdf.generation_timeout_seconds", 30)?
        .set_default("cdn.enabled", false)?
        .set_default("cdn.base_url", "")?
        .set_default("cdn.api_key", "")?
        .set_default("cdn.bucket_name", "")?
        .set_default("cdn.upload_timeout_seconds", 30)?
        .set_default("file_storage.upload_dir", "./uploads")?
        .set_default("file_storage.max_file_size", 104_857_600_i64)?
        .build()?;

    let app_config: AppConfig = config.try_deserialize()?;
    Ok(app_config)
}

async fn init_server(config: AppConfig) -> Result<AppState> {
    info!("Initializing Adeptus API server");

    let db = DatabaseManager::new(
        &config.database.url,
        config.database.max_connections,
        config.database.min_connections,
    )
    .await?;

    if config.database.run_migrations {
        info!("Running database migrations");
        db.run_migrations().await?;
        info!("Database migrations completed");
    }

    let repos = RepositoryManager::new(db.pool().clone());

    let events = EventPublisher::new(
        config.nats.url.as_deref(),
        config.nats.stream_name.clone(),
        config.nats.max_age_days,
    )
    .await?;

    let keto = KetoClient::new(config.keto.read_url.clone());
    let pactum = PactumClient::new(&config.pactum);

    Ok(AppState {
        db,
        repos,
        config,
        events,
        keto,
        pactum,
    })
}

fn create_router(state: AppState, prometheus_registry: prometheus::Registry) -> Router {
    let schema = build_schema(state.clone());
    let rate_limit_state = rate_limit::RateLimitState::new(&state.config.rate_limiting);
    let timeout_seconds = state.config.server.timeout_seconds;

    let health_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .with_state(state.clone());

    let graphql_routes = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .with_state(schema);

    let file_routes = Router::new()
        .route("/api/files/{file_id}/download", get(file::download_file))
        .route(
            "/api/documents/{document_id}/files/upload",
            post(file::upload_file),
        )
        .with_state(state.clone());

    let pdf_routes = Router::new()
        .route(
            "/api/documents/{document_id}/pdf",
            get(pdf::generate_pdf_handler),
        )
        .route(
            "/api/documents/{document_id}/version",
            get(pdf::document_version_handler),
        )
        .with_state(state);

    let metrics_routes = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(prometheus_registry);

    health_routes
        .merge(graphql_routes)
        .merge(file_routes)
        .merge(pdf_routes)
        .merge(metrics_routes)
        .layer(axum_middleware::from_fn_with_state(
            rate_limit_state,
            rate_limit::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn(auth::extract_subject))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(timeout_seconds),
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()))
}

async fn metrics_handler(
    axum::extract::State(registry): axum::extract::State<prometheus::Registry>,
) -> Result<String, StatusCode> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    String::from_utf8(buffer).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn run_server(config: AppConfig) -> Result<()> {
    let obs = observability::init(&config.observability)?;

    info!("Starting Adeptus v{}", env!("CARGO_PKG_VERSION"));

    let state = match init_server(config).await {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    let addr = SocketAddr::new(state.config.server.host.parse()?, state.config.server.port);

    let app = create_router(state, obs.prometheus_registry.clone());

    info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            error!("Server error: {}", e);
            e
        })?;

    observability::shutdown(&obs);
    info!("Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = load_config()?;
    run_server(config).await
}
