use std::env;

/// Runtime configuration read from the environment once at startup, matching the
/// `SHARED_ASSETS_URL` / `INGRESS_BASE_PATH` convention documented in
/// `docs/frontend-conventions.md` (sweetrpg/platform).
pub struct Config {
    pub port: u16,
    pub shared_assets_url: String,
    pub otlp_endpoint: Option<String>,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            shared_assets_url: env::var("SHARED_ASSETS_URL")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }
}
