use std::env;

/// Runtime configuration read from the environment once at startup, matching the
/// `SHARED_ASSETS_URL` / `INGRESS_BASE_PATH` convention documented in
/// `docs/frontend-conventions.md` (sweetrpg/platform).
pub struct Config {
    pub port: u16,
    pub shared_assets_url: String,
    pub otlp_endpoint: Option<String>,
    pub log_level: String,
    /// Base URL for `admin-api` (banner messages). Unset by default - the `AdminClient`
    /// stays disabled (no network calls, always returns no banners) until this is
    /// explicitly set, so the deploy is inert until rollout enables it.
    pub admin_api_url: Option<String>,
    /// Host of the shared `redis.sweetrpg-support` instance (not dedicated to this app - see
    /// `docs/frontend-conventions.md`'s "Shared sweetrpg-support Redis instance" section for
    /// the DB-index registry). Unset by default - the `SessionClient` stays disabled (every
    /// visitor reads as logged-out) until this is set.
    pub redis_host: Option<String>,
    pub redis_port: u16,
    /// Logical DB index within that shared instance - must match `auth-web`'s own, since this
    /// app reads the exact session keys `auth-web` writes.
    pub redis_db: u8,
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
            admin_api_url: env::var("ADMIN_API_URL").ok(),
            redis_host: env::var("REDIS_HOST").ok().filter(|v| !v.is_empty()),
            redis_port: env::var("REDIS_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(6379),
            redis_db: env::var("REDIS_DB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
        }
    }
}
