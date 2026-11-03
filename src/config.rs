use std::env;

use crate::constants::SUPPORT_CONTACT_ENV;

/// Runtime configuration read from the environment once at startup, matching the
/// `SHARED_URL` / `INGRESS_BASE_PATH` convention documented in
/// `docs/frontend-conventions.md` (sweetrpg/platform).
pub struct Config {
    pub port: u16,
    pub shared_url: String,
    // pub assets_url: String,
    pub otlp_endpoint: Option<String>,
    pub log_level: String,
    /// Base URL for `admin-api` (banner messages). Unset by default - the `AdminClient`
    /// stays disabled (no network calls, always returns no banners) until this is
    /// explicitly set, so the deploy is inert until rollout enables it.
    pub admin_api_url: Option<String>,
    /// URL for `admin-api`'s feedback endpoint, used by the browser-side feedback widget's
    /// `data-api-url` attribute - distinct from `admin_api_url`, which is this service's own
    /// server-side in-cluster call and never reaches the browser. Defaults to a path-only
    /// value: every API on this platform is exposed under the *same* `dev.sweetrpg.com` host
    /// as every frontend, at `/api/0/<package>` (see `docs/deployment-conventions.md`'s "API
    /// Ingress path versioning") - so this call is same-origin from the browser's perspective,
    /// not cross-origin, and needs no host/scheme of its own.
    pub feedback_api_url: String,
    /// Host of the shared session Redis instance `auth-web` owns - this app only ever reads
    /// it, never writes. Unset by default - the `SessionClient` stays disabled (every visitor
    /// reads as logged-out) until this is set. Named `SHARED_SESSION_REDIS_*`, matching
    /// `catalog-web`'s convention, to leave `REDIS_*` free for this app's own cache/rate-limit
    /// Redis if one is ever added - the two must never collide on the same env var names.
    pub shared_session_redis_host: Option<String>,
    pub shared_session_redis_port: u16,
    /// Logical DB index on the shared Redis instance. Must match `auth-web`'s own `REDIS_DB`
    /// env var value (DB 2 in every deployed environment) - `auth-web` is the sole writer, so
    /// reading from
    /// the wrong DB index silently degrades to "every visitor reads as logged-out" the same
    /// way an unreachable host does.
    pub shared_session_redis_db: u16,
    /// Password for the shared session Redis instance. Unset by default, matching an
    /// unauthenticated Redis (fine for some local setups); every deployed environment's
    /// shared Redis requires auth, so this must be set there or reads fail with `NOAUTH` and
    /// degrade to "every visitor reads as logged-out" the same way an unreachable host does.
    pub shared_session_redis_pass: Option<String>,
    /// Sentry error-reporting DSN. Unset by default - reporting stays a no-op (not a startup
    /// failure) until this is set, e.g. local dev, where Sentry isn't used at all.
    pub sentry_dsn: Option<String>,
    pub env: String,
    /// Where users reach support: an email address or an http(s) URL. Required - the
    /// `/support` page has nothing to show without it, so a missing or malformed value stops
    /// startup rather than shipping a page with no way to get help.
    pub support_contact: SupportContact,
}

/// Validated support contact, ready for the `/support` page to link to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportContact {
    /// Shown as the link text.
    pub display: String,
    /// The link target (`mailto:` for an email address, the URL itself otherwise).
    pub href: String,
}

impl SupportContact {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let value = raw.trim();
        if value.is_empty() || value.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(format!(
                "{SUPPORT_CONTACT_ENV} must be a single non-empty value"
            ));
        }
        let is_url = value.starts_with("https://") || value.starts_with("http://");
        let is_email = value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
        let href = match (is_url, is_email) {
            (true, _) => value.to_string(),
            (false, true) => format!("mailto:{value}"),
            (false, false) => {
                return Err(format!(
                    "{SUPPORT_CONTACT_ENV} must be an email address or an http(s) URL"
                ))
            }
        };
        Ok(Self {
            display: value.to_string(),
            href,
        })
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            shared_url: env::var("SHARED_URL")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            // assets_url: env::var("ASSETS_URL")
            //     .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            admin_api_url: env::var("ADMIN_API_URL").ok(),
            feedback_api_url: env::var("FEEDBACK_API_URL")
                .unwrap_or_else(|_| "/api/0/admin/feedback".to_string()),
            shared_session_redis_host: env::var("SHARED_SESSION_REDIS_HOST")
                .ok()
                .filter(|v| !v.is_empty()),
            shared_session_redis_port: env::var("SHARED_SESSION_REDIS_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(6379),
            shared_session_redis_db: env::var("SHARED_SESSION_REDIS_DB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            shared_session_redis_pass: env::var("SHARED_SESSION_REDIS_PASS")
                .ok()
                .filter(|v| !v.is_empty()),
            sentry_dsn: env::var("SENTRY_DSN").ok().filter(|v| !v.is_empty()),
            env: env::var("ENV").unwrap_or_else(|_| "dev".to_string()),
            support_contact: env::var(SUPPORT_CONTACT_ENV)
                .map_err(|_| format!("{SUPPORT_CONTACT_ENV} is not set"))
                .and_then(|v| SupportContact::parse(&v))
                .unwrap_or_else(|err| panic!("invalid configuration: {err}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_becomes_a_mailto_link() {
        let contact = SupportContact::parse(" help@example.com ").unwrap();
        assert_eq!(contact.display, "help@example.com");
        assert_eq!(contact.href, "mailto:help@example.com");
    }

    #[test]
    fn http_url_is_used_as_is() {
        let contact = SupportContact::parse("https://example.com/help").unwrap();
        assert_eq!(contact.href, "https://example.com/help");
    }

    #[test]
    fn invalid_values_are_rejected() {
        for bad in [
            "",
            "   ",
            "help",
            "@example.com",
            "a@b",
            "javascript:alert(1)",
            "two words@example.com",
        ] {
            assert!(
                SupportContact::parse(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }
}
