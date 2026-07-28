use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;

/// A banner message as returned by `admin-api`'s `GET /banners`. Only the fields this
/// service renders are deserialized - `admin-api`'s `models.BannerMessage` has more
/// (id, scope, timestamps) that main-web has no use for yet.
#[derive(Clone, Debug, Deserialize)]
pub struct Banner {
    pub severity: String,
    pub message: String,
}

struct CacheEntry {
    fetched_at: Instant,
    banners: Vec<Banner>,
}

/// Fetches banner messages from `admin-api` for this service's scopes, with a bounded-TTL
/// cache and fail-open behavior: any error, timeout, or missing `ADMIN_API_URL` yields an
/// empty list rather than surfacing a failure to the page being decorated.
pub struct AdminClient {
    http: reqwest::Client,
    base_url: Option<String>,
    ttl: Duration,
    cache: Mutex<Option<CacheEntry>>,
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
            cache: Mutex::new(None),
        }
    }

    /// Returns active banners for the given scopes (e.g. `["platform", "service:main"]`),
    /// most-severe-first as returned by `admin-api`. Never errors: any failure (disabled
    /// client, timeout, network error, non-200, bad body) yields an empty `Vec`.
    pub async fn fetch_banners(&self, scopes: &[&str]) -> Vec<Banner> {
        let Some(base_url) = &self.base_url else {
            return Vec::new();
        };

        if let Some(cached) = self.cached() {
            return cached;
        }

        let mut url = match reqwest::Url::parse(&format!("{base_url}/banners")) {
            Ok(url) => url,
            Err(err) => {
                tracing::warn!(error = %err, "invalid ADMIN_API_URL, skipping banner fetch");
                return Vec::new();
            }
        };
        {
            let mut pairs = url.query_pairs_mut();
            for scope in scopes {
                pairs.append_pair("scope", scope);
            }
        }

        let banners = match self.http.get(url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.json::<Vec<Banner>>().await {
                Ok(banners) => banners,
                Err(err) => {
                    tracing::warn!(error = %err, "failed to decode admin-api banner response");
                    Vec::new()
                }
            },
            Ok(resp) => {
                tracing::warn!(status = %resp.status(), "admin-api returned a non-success status");
                Vec::new()
            }
            Err(err) => {
                tracing::warn!(error = %err, "admin-api request failed or timed out");
                Vec::new()
            }
        };

        self.store(banners.clone());
        banners
    }

    fn cached(&self) -> Option<Vec<Banner>> {
        let guard = self.cache.lock().ok()?;
        let entry = guard.as_ref()?;
        if entry.fetched_at.elapsed() < self.ttl {
            Some(entry.banners.clone())
        } else {
            None
        }
    }

    fn store(&self, banners: Vec<Banner>) {
        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(CacheEntry {
                fetched_at: Instant::now(),
                banners,
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
}
