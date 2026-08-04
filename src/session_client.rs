use std::collections::HashMap;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::Deserialize;

/// Shared across every frontend - see design.md's "One shared cookie" decision in the
/// add-user-api-authn-authz OpenSpec change. `auth-web` is the only writer; this app only
/// reads it.
pub const SESSION_COOKIE_NAME: &str = "sweetrpg_session";

/// The identity `auth-web` writes into the shared session store - mirrors its own
/// `SessionUser` model. `roles` comes from `users-api`'s verified `/authz/check` response, not
/// an unverified local decode.
#[derive(Clone, Debug, Deserialize)]
pub struct SessionUser {
    #[allow(dead_code)]
    pub sub: String,
    pub name: String,
    #[allow(dead_code)]
    pub email: Option<String>,
    pub roles: Vec<String>,
}

/// Reads (never writes) the shared session `auth-web` establishes. Fails open: any error
/// (disabled, unreachable Redis, missing/malformed session) is treated as "no session" rather
/// than surfacing a failure to the page being decorated - the same fail-open contract Vapor's
/// `ResilientRedisSessionDriver` gives every Swift frontend, applied here since main-web isn't
/// Vapor and has no session driver of its own to plug into.
pub struct SessionClient {
    manager: Option<ConnectionManager>,
}

impl SessionClient {
    /// `host` is `None` when `SHARED_SESSION_REDIS_HOST` is unset - the client is then
    /// permanently disabled (every lookup immediately returns `None`, no network calls at all).
    /// `password` is `None` when `SHARED_SESSION_REDIS_PASS` is unset, matching an
    /// unauthenticated Redis instance (e.g. some local setups) - matches `catalog-web`'s
    /// `SHARED_SESSION_REDIS_PASS` convention (`configure.swift`), which this app previously
    /// had no equivalent of at all.
    pub async fn new(host: Option<String>, port: u16, db: u16, password: Option<String>) -> Self {
        let Some(host) = host else {
            tracing::warn!("SHARED_SESSION_REDIS_HOST not set - shared session reads disabled, every visitor treated as logged-out");
            return Self { manager: None };
        };

        let credentials = password
            .as_deref()
            .map(|pass| format!(":{pass}@"))
            .unwrap_or_default();
        let url = format!("redis://{credentials}{host}:{port}/{db}");
        let manager = match redis::Client::open(url) {
            Ok(client) => match ConnectionManager::new(client).await {
                Ok(manager) => Some(manager),
                Err(err) => {
                    tracing::warn!(error = %err, "failed to connect to shared session Redis, sessions will read as logged-out");
                    None
                }
            },
            Err(err) => {
                tracing::warn!(error = %err, "invalid SHARED_SESSION_REDIS_HOST, sessions will read as logged-out");
                None
            }
        };
        Self { manager }
    }

    /// Looks up the session for a session ID (the shared cookie's raw value). Never errors -
    /// any failure (disabled client, unreachable Redis, missing key, malformed JSON) degrades
    /// to `None`.
    pub async fn current_user(&self, session_id: &str) -> Option<SessionUser> {
        let mut manager = self.manager.clone()?;

        // Key format matches Vapor's ResilientRedisSessionDriver (auth-web's session writer):
        // `vrs-<sessionID>`, JSON-encoded Vapor `SessionData` (a flat `[String: String]`).
        let key = format!("vrs-{session_id}");
        let raw: Option<String> = match manager.get(&key).await {
            Ok(raw) => raw,
            Err(err) => {
                tracing::warn!(error = %err, "shared session Redis read failed, treating as logged-out");
                return None;
            }
        };
        let raw = raw?;

        let session_data: HashMap<String, String> = match serde_json::from_str(&raw) {
            Ok(data) => data,
            Err(err) => {
                tracing::warn!(error = %err, "malformed shared session data, treating as logged-out");
                return None;
            }
        };
        let user_json = session_data.get("user")?;
        match serde_json::from_str(user_json) {
            Ok(user) => Some(user),
            Err(err) => {
                tracing::warn!(error = %err, "malformed shared session user, treating as logged-out");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disabled_client_returns_no_user_without_making_a_request() {
        let client = SessionClient::new(None, 6379, 0, None).await;
        assert!(client.current_user("any-session-id").await.is_none());
    }

    #[tokio::test]
    async fn unreachable_redis_fails_open() {
        // Port 1 is a reserved/unassigned port that refuses connections immediately on any
        // platform this runs on - exercises the connection-failure branch without a mock
        // server or real timeout wait.
        let client = SessionClient::new(Some("127.0.0.1".to_string()), 1, 0, None).await;
        assert!(client.current_user("any-session-id").await.is_none());
    }

    #[tokio::test]
    async fn unreachable_redis_fails_open_with_a_password_configured() {
        // Exercises the credentialed URL construction path (unreachable, so no real auth
        // happens) without needing a real authenticated Redis instance in tests.
        let client = SessionClient::new(
            Some("127.0.0.1".to_string()),
            1,
            0,
            Some("s3cret".to_string()),
        )
        .await;
        assert!(client.current_user("any-session-id").await.is_none());
    }
}
