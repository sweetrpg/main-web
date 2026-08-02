use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::admin_client::Banner;
use crate::session_client::SESSION_COOKIE_NAME;
use crate::AppState;

/// The suite's app list is a compile-time data structure, not a config file — see
/// `design.md`'s "Decisions" section in the `main-web-rust-rewrite` OpenSpec change for why a
/// YAML/JSON config was rejected (YAGNI: adding a new app already requires a code change and
/// redeploy to update its `href`).
struct AppCard {
    name: &'static str,
    description: &'static str,
    href: &'static str,
    has_status: bool,
    status: &'static str,
    background: &'static str,
}

fn apps() -> Vec<AppCard> {
    vec![
        AppCard {
            name: "Catalogue",
            description: "Browse, rate and review every RPG book in print.",
            href: "/catalog",
            has_status: false,
            status: "",
            background: "catalog-card-back.png",
        },
        AppCard {
            name: "Shelf",
            description: "Track what you own, want, and are playing right now.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "shelf-card-back.jpg",
        },
        AppCard {
            name: "Systems",
            description: "Deep-dive reference on the game systems behind the books.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "systems-card-back.jpg",
        },
        AppCard {
            name: "Profile",
            description: "Your account, your table, your reading history.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "profile-card-back.jpg",
        },
        AppCard {
            name: "Initiative!",
            description: "Track turn order and initiative live at the table.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "initiative-card-back.png",
        },
    ]
}

#[derive(Template)]
#[template(path = "main.html")]
struct LandingTemplate {
    shared_assets_url: String,
    apps: Vec<AppCard>,
    version: String,
    build_date: String,
    build_hash: String,
    banners: Vec<Banner>,
    /// `Some(display_name)` when the shared session (see `session_client`) resolves to a
    /// logged-in visitor; `None` otherwise. `auth-web` is the only writer of this session -
    /// this app only ever reads it.
    current_user_name: Option<String>,
    login_url: String,
    logout_url: String,
}

impl IntoResponse for LandingTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(body) => Html(body).into_response(),
            Err(err) => {
                tracing::error!(error = %err, "failed to render landing page template");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[derive(Deserialize)]
struct IndexQuery {
    login_error: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(index))
}

async fn index(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IndexQuery>,
    jar: CookieJar,
) -> LandingTemplate {
    let mut banners = state
        .admin_client
        .fetch_banners(&["platform", "service:main"])
        .await;

    // auth-web (the suite's sole Auth0 callback handler) redirects back here with
    // ?login_error=1 on any login failure - render it via the same banner markup admin-api's
    // banners use, rather than a bespoke alert element, so it gets the existing styling/theming
    // for free. Synthesized client-side, not from admin-api, so it's never cached or shared
    // with other visitors.
    if query.login_error.is_some() {
        banners.insert(
            0,
            Banner {
                severity: "critical".to_string(),
                message: "Login failed. Please try again.".to_string(),
            },
        );
    }

    let current_user_name = match jar.get(SESSION_COOKIE_NAME) {
        Some(cookie) => state
            .session_client
            .current_user(cookie.value())
            .await
            .map(|user| user.name),
        None => None,
    };

    // No INGRESS_BASE_PATH prefix needed, unlike catalog-web/admin-web: auth-web sits at
    // /auth on this same host, and main-web itself serves the host's root - see
    // design.md's "auth-web is the sole owner of the Authorization Code exchange" decision.
    LandingTemplate {
        shared_assets_url: state.config.shared_assets_url.clone(),
        apps: apps(),
        version: state.build_info.version.clone(),
        build_date: state.build_info.date.clone(),
        // Pre-truncated here rather than sliced in the template: a real commit SHA can be
        // longer than 8 chars, and Askama's `build_hash[..8]` panicked on any string shorter
        // than 8 bytes (the local-dev placeholder is exactly 8, but don't rely on that).
        build_hash: state
            .build_info
            .sha
            .get(..8)
            .unwrap_or(&state.build_info.sha)
            .to_string(),
        banners,
        current_user_name,
        login_url: "/auth/login?return_to=/".to_string(),
        logout_url: "/auth/logout".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(current_user_name: Option<String>) -> LandingTemplate {
        LandingTemplate {
            shared_assets_url: "http://localhost:8081".to_string(),
            apps: apps(),
            version: "dev".to_string(),
            build_date: "unset".to_string(),
            build_hash: "unset".to_string(),
            banners: Vec::new(),
            current_user_name,
            login_url: "/auth/login?return_to=/".to_string(),
            logout_url: "/auth/logout".to_string(),
        }
    }

    #[test]
    fn logged_out_shows_log_in_link() {
        let html = template(None).render().expect("template renders");
        assert!(html.contains(r#"href="/auth/login?return_to=/""#));
        assert!(html.contains("Log in"));
        assert!(!html.contains("Log out"));
    }

    #[test]
    fn login_error_query_parses_from_form_encoded_flag() {
        let query: IndexQuery = serde_urlencoded::from_str("login_error=1").unwrap();
        assert_eq!(query.login_error.as_deref(), Some("1"));

        let query: IndexQuery = serde_urlencoded::from_str("").unwrap();
        assert_eq!(query.login_error, None);
    }

    #[test]
    fn login_error_banner_renders_as_critical() {
        let mut tpl = template(None);
        tpl.banners.push(Banner {
            severity: "critical".to_string(),
            message: "Login failed. Please try again.".to_string(),
        });
        let html = tpl.render().expect("template renders");
        assert!(html.contains("banner-critical"));
        assert!(html.contains("Login failed. Please try again."));
    }

    #[test]
    fn logged_in_shows_name_and_log_out_form() {
        let html = template(Some("Alice".to_string()))
            .render()
            .expect("template renders");
        assert!(html.contains("Alice"));
        assert!(html.contains(r#"action="/auth/logout""#));
        assert!(html.contains("Log out"));
        assert!(!html.contains(">Log in<"));
    }
}
