mod build_info;
mod config;
mod routes;
mod telemetry;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use axum_prometheus::PrometheusMetricLayer;
use build_info::BuildInfo;
use config::Config;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub struct AppState {
    config: Config,
    build_info: BuildInfo,
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let tracer_provider = telemetry::init(&config);

    let build_info = BuildInfo::load();
    let port = config.port;

    let addr = format!("0.0.0.0:{port}");
    tracing::info!(
        %addr,
        version = %build_info.version,
        sha = %build_info.sha,
        number = %build_info.number,
        job = %build_info.job,
        "starting sweetrpg-main-web"
    );

    let state = Arc::new(AppState { config, build_info });

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app = Router::new()
        .merge(routes::hub::router())
        .with_state(state)
        .merge(routes::health::router())
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .nest_service("/static", ServeDir::new("static"))
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");

    if let Some(provider) = tracer_provider {
        let _ = provider.shutdown();
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("shutdown signal received");
}
