use std::sync::Mutex;
use std::time::{Duration, Instant};

use opentelemetry::propagation::Injector;
use serde::Deserialize;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Lets the global propagator write traceparent/tracestate onto a `reqwest` request's headers -
/// `reqwest::header::HeaderMap` has no built-in `opentelemetry::propagation::Injector` impl.
struct HeaderInjector<'a>(&'a mut reqwest::header::HeaderMap);

impl Injector for HeaderInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(name), Ok(val)) = (
            reqwest::header::HeaderName::from_bytes(key.as_bytes()),
            reqwest::header::HeaderValue::from_str(&value),
        ) {
            self.0.insert(name, val);
        }
    }
}

/// A banner message as returned by `admin-api`'s `GET /banners`. Only the fields this
/// service renders are deserialized - `admin-api`'s `models.BannerMessage` has more
/// (id, scope, timestamps) that main-web has no use for yet.
#[derive(Clone, Debug, Deserialize)]
pub struct Banner {
    pub severity: String,
    pub message: String,
}

/// A maintenance-mode record as returned by `admin-api`'s `GET /maintenance-modes/active`.
/// Only the fields this service renders (the badge tooltip) are deserialized.
#[derive(Clone, Debug, Deserialize)]
pub struct MaintenanceMode {
    pub scope_type: String,
    pub scope_value: String,
    pub label: String,
    pub description: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
}

/// An app-card-status record as returned by `admin-api`'s `GET /app-card-statuses/active`.
/// Only the fields this service renders are deserialized.
#[derive(Clone, Debug, Deserialize)]
pub struct AppCardStatus {
    pub scope_type: String,
    pub scope_value: String,
    pub label: String,
}

struct CacheEntry<T> {
    fetched_at: Instant,
    value: T,
}

/// Fetches banner messages, maintenance-mode state, and app-card status from `admin-api` for
/// this service's scopes, with a bounded-TTL cache and fail-open behavior: any error, timeout,
/// or missing `ADMIN_API_URL` yields an empty list rather than surfacing a failure to the page
/// being decorated.
pub struct AdminClient {
    http: reqwest::Client,
    base_url: Option<String>,
    ttl: Duration,
    banner_cache: Mutex<Option<CacheEntry<Vec<Banner>>>>,
    maintenance_cache: Mutex<Option<CacheEntry<Vec<MaintenanceMode>>>>,
    app_card_status_cache: Mutex<Option<CacheEntry<Vec<AppCardStatus>>>>,
}

impl AdminClient {
    /// `base_url` is `None` when `ADMIN_API_URL` is unset - the client is then permanently
    /// disabled (every fetch immediately returns an empty list, no network calls at all),
    /// matching the "unset/disabled by default" rollout requirement.
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url,
            ttl: Duration::from_secs(90),
            banner_cache: Mutex::new(None),
            maintenance_cache: Mutex::new(None),
            app_card_status_cache: Mutex::new(None),
        }
    }

    /// Returns active banners for the given scopes (e.g. `["platform", "service:main"]`),
    /// most-severe-first as returned by `admin-api`. Never errors: any failure (disabled
    /// client, timeout, network error, non-200, bad body) yields an empty `Vec`.
    pub async fn fetch_banners(&self, scopes: &[&str]) -> Vec<Banner> {
        let Some(base_url) = &self.base_url else {
            return Vec::new();
        };

        if let Some(cached) = Self::cached(&self.banner_cache, self.ttl) {
            return cached;
        }

        let banners = self.get_scoped(base_url, "banners", scopes, "banner").await;

        Self::store(&self.banner_cache, banners.clone());
        banners
    }

    /// Returns active maintenance-mode records for the given scopes (e.g.
    /// `["platform", "service:catalog"]`). Never errors: any failure (disabled client,
    /// timeout, network error, non-200, bad body) yields an empty `Vec`, so an `admin-api`
    /// outage never turns into a false "under maintenance" state.
    pub async fn fetch_maintenance_modes(&self, scopes: &[&str]) -> Vec<MaintenanceMode> {
        let Some(base_url) = &self.base_url else {
            return Vec::new();
        };

        if let Some(cached) = Self::cached(&self.maintenance_cache, self.ttl) {
            return cached;
        }

        let modes = self
            .get_scoped(
                base_url,
                "maintenance-modes/active",
                scopes,
                "maintenance-mode",
            )
            .await;

        Self::store(&self.maintenance_cache, modes.clone());
        modes
    }

    /// Returns active app-card-status records for the given scopes (e.g.
    /// `["service:catalog", "service:game_room"]`). Never errors: any failure (disabled client,
    /// timeout, network error, non-200, bad body) yields an empty `Vec`.
    pub async fn fetch_app_card_statuses(&self, scopes: &[&str]) -> Vec<AppCardStatus> {
        let Some(base_url) = &self.base_url else {
            return Vec::new();
        };

        if let Some(cached) = Self::cached(&self.app_card_status_cache, self.ttl) {
            return cached;
        }

        let statuses = self
            .get_scoped(
                base_url,
                "app-card-statuses/active",
                scopes,
                "app-card-status",
            )
            .await;

        Self::store(&self.app_card_status_cache, statuses.clone());
        statuses
    }

    async fn get_scoped<T: serde::de::DeserializeOwned>(
        &self,
        base_url: &str,
        path: &str,
        scopes: &[&str],
        kind: &str,
    ) -> Vec<T> {
        let mut url = match reqwest::Url::parse(&format!("{base_url}/{path}")) {
            Ok(url) => url,
            Err(err) => {
                tracing::warn!(error = %err, kind, "invalid ADMIN_API_URL, skipping fetch");
                return Vec::new();
            }
        };
        {
            let mut pairs = url.query_pairs_mut();
            for scope in scopes {
                pairs.append_pair("scope", scope);
            }
        }

        let mut headers = reqwest::header::HeaderMap::new();
        let otel_context = tracing::Span::current().context();
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&otel_context, &mut HeaderInjector(&mut headers))
        });

        match self.http.get(url).headers(headers).send().await {
            Ok(resp) if resp.status().is_success() => match resp.json::<Vec<T>>().await {
                Ok(items) => items,
                Err(err) => {
                    tracing::warn!(error = %err, kind, "failed to decode admin-api response");
                    Vec::new()
                }
            },
            Ok(resp) => {
                tracing::warn!(status = %resp.status(), kind, "admin-api returned a non-success status");
                Vec::new()
            }
            Err(err) => {
                tracing::warn!(error = %err, kind, "admin-api request failed or timed out");
                Vec::new()
            }
        }
    }

    fn cached<T: Clone>(cache: &Mutex<Option<CacheEntry<T>>>, ttl: Duration) -> Option<T> {
        let guard = cache.lock().ok()?;
        let entry = guard.as_ref()?;
        if entry.fetched_at.elapsed() < ttl {
            Some(entry.value.clone())
        } else {
            None
        }
    }

    fn store<T>(cache: &Mutex<Option<CacheEntry<T>>>, value: T) {
        if let Ok(mut guard) = cache.lock() {
            *guard = Some(CacheEntry {
                fetched_at: Instant::now(),
                value,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disabled_client_returns_no_banners_without_making_a_request() {
        let client = AdminClient::new(None);
        let banners = client.fetch_banners(&["platform"]).await;
        assert!(banners.is_empty());
    }

    #[tokio::test]
    async fn unreachable_admin_api_fails_open() {
        // Port 1 is a reserved/unassigned port that refuses connections immediately on
        // any platform this runs on - exercises the network-error branch without a mock
        // server or real timeout wait.
        let client = AdminClient::new(Some("http://127.0.0.1:1".to_string()));
        let banners = client.fetch_banners(&["platform"]).await;
        assert!(banners.is_empty());
    }

    #[tokio::test]
    async fn disabled_client_returns_no_maintenance_modes_without_making_a_request() {
        let client = AdminClient::new(None);
        let modes = client.fetch_maintenance_modes(&["platform"]).await;
        assert!(modes.is_empty());
    }

    #[tokio::test]
    async fn unreachable_admin_api_fails_open_for_maintenance_modes() {
        let client = AdminClient::new(Some("http://127.0.0.1:1".to_string()));
        let modes = client.fetch_maintenance_modes(&["platform"]).await;
        assert!(modes.is_empty());
    }

    #[tokio::test]
    async fn disabled_client_returns_no_app_card_statuses_without_making_a_request() {
        let client = AdminClient::new(None);
        let statuses = client.fetch_app_card_statuses(&["service:catalog"]).await;
        assert!(statuses.is_empty());
    }

    #[tokio::test]
    async fn unreachable_admin_api_fails_open_for_app_card_statuses() {
        let client = AdminClient::new(Some("http://127.0.0.1:1".to_string()));
        let statuses = client.fetch_app_card_statuses(&["service:catalog"]).await;
        assert!(statuses.is_empty());
    }
}
