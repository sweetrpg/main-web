use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::admin_client::{Banner, MaintenanceMode};
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
    /// The `service:<name>` scope this card's app registers with `admin-api`, or `None` for
    /// a card with no backing service yet (e.g. "Systems") - such a card never shows a
    /// maintenance badge since no scope exists to query.
    service_scope: Option<&'static str>,
    /// Populated after `apps()` returns, from the platform + this card's own service scope's
    /// active maintenance-mode records - not compile-time data like the fields above.
    maintenance: Option<MaintenanceBadge>,
}

/// The maintenance-mode content an app card's badge tooltip renders, resolved from whichever
/// active record applies (the card's own service scope takes precedence over the platform
/// scope when both are active, since it's the more specific message).
struct MaintenanceBadge {
    label: String,
    description: String,
    starts_at: String,
    ends_at: Option<String>,
}

impl From<&MaintenanceMode> for MaintenanceBadge {
    fn from(mode: &MaintenanceMode) -> Self {
        Self {
            label: mode.label.clone(),
            description: mode.description.clone(),
            starts_at: mode.starts_at.clone(),
            ends_at: mode.ends_at.clone(),
        }
    }
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
            service_scope: Some("service:catalog"),
            maintenance: None,
        },
        AppCard {
            name: "Shelf",
            description: "Track what you own, want, and are playing right now.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "shelf-card-back.jpg",
            service_scope: Some("service:shelf"),
            maintenance: None,
        },
        AppCard {
            name: "Systems",
            description: "Deep-dive reference on the game systems behind the books.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "systems-card-back.jpg",
            service_scope: None,
            maintenance: None,
        },
        AppCard {
            name: "Profile",
            description: "Your account, your table, your reading history.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "profile-card-back.jpg",
            service_scope: Some("service:users"),
            maintenance: None,
        },
        AppCard {
            name: "Initiative!",
            description: "Track turn order and initiative live at the table.",
            href: "#",
            has_status: true,
            status: "Coming soon",
            background: "initiative-card-back.png",
            service_scope: Some("service:initiative"),
            maintenance: None,
        },
    ]
}

/// Resolves each card's `maintenance` field from the active maintenance-mode records
/// fetched for `["platform", <card's own service scope>, ...]`. A card's own service-scoped
/// record takes precedence over a platform-wide one when both are active.
fn apply_maintenance(apps: &mut [AppCard], records: &[MaintenanceMode]) {
    let platform_record = records.iter().find(|m| m.scope_type == "platform");
    for app in apps.iter_mut() {
        let Some(scope) = app.service_scope else {
            continue;
        };
        let service_value = scope.trim_start_matches("service:");
        let service_record = records
            .iter()
            .find(|m| m.scope_type == "service" && m.scope_value == service_value);
        app.maintenance = service_record
            .or(platform_record)
            .map(MaintenanceBadge::from);
    }
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

    let mut apps = apps();
    let mut maintenance_scopes = vec!["platform"];
    maintenance_scopes.extend(apps.iter().filter_map(|app| app.service_scope));
    let maintenance_records = state
        .admin_client
        .fetch_maintenance_modes(&maintenance_scopes)
        .await;
    apply_maintenance(&mut apps, &maintenance_records);

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
        apps,
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

    fn maintenance_mode(scope_type: &str, scope_value: &str) -> MaintenanceMode {
        MaintenanceMode {
            scope_type: scope_type.to_string(),
            scope_value: scope_value.to_string(),
            label: "Scheduled downtime".to_string(),
            description: "Upgrading the database.".to_string(),
            starts_at: "2026-08-01T00:00:00Z".to_string(),
            ends_at: Some("2026-08-01T04:00:00Z".to_string()),
        }
    }

    #[test]
    fn service_scoped_maintenance_affects_only_that_card() {
        let mut apps = apps();
        apply_maintenance(&mut apps, &[maintenance_mode("service", "catalog")]);

        let catalog = apps.iter().find(|a| a.name == "Catalogue").unwrap();
        assert!(catalog.maintenance.is_some());
        let others_unaffected = apps
            .iter()
            .filter(|a| a.name != "Catalogue")
            .all(|a| a.maintenance.is_none());
        assert!(others_unaffected);
    }

    #[test]
    fn platform_scoped_maintenance_affects_every_card_with_a_service_scope() {
        let mut apps = apps();
        apply_maintenance(&mut apps, &[maintenance_mode("platform", "")]);

        for app in &apps {
            assert_eq!(app.maintenance.is_some(), app.service_scope.is_some());
        }
    }

    #[test]
    fn maintenance_badge_renders_with_tooltip() {
        let mut tpl = template(None);
        tpl.apps[0].maintenance = Some(MaintenanceBadge {
            label: "Scheduled downtime".to_string(),
            description: "Upgrading the database.".to_string(),
            starts_at: "2026-08-01T00:00:00Z".to_string(),
            ends_at: Some("2026-08-01T04:00:00Z".to_string()),
        });
        let html = tpl.render().expect("template renders");
        assert!(html.contains("tag-danger"));
        assert!(html.contains("Scheduled downtime: Upgrading the database."));
        assert!(html.contains("2026-08-01T00:00:00Z - 2026-08-01T04:00:00Z"));
    }
}
