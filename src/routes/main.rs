use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::admin_client::{Banner, MaintenanceMode};
use crate::i18n::Tr;
use crate::session_client::SESSION_COOKIE_NAME;
use crate::AppState;

/// The suite's app list is a compile-time data structure, not a config file — see
/// `design.md`'s "Decisions" section in the `main-web-rust-rewrite` OpenSpec change for why a
/// YAML/JSON config was rejected (YAGNI: adding a new app already requires a code change and
/// redeploy to update its `href`).
struct AppCard {
    name: String,
    description: String,
    href: &'static str,
    has_status: bool,
    status: String,
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

fn apps(tr: &Tr) -> Vec<AppCard> {
    let coming_soon = tr.cards_coming_soon();
    vec![
        AppCard {
            name: tr.get("cards.catalogue.name"),
            description: tr.get("cards.catalogue.description"),
            href: "/catalog",
            has_status: false,
            status: String::new(),
            background: "catalog-card-back.png",
            service_scope: Some("service:catalog"),
            maintenance: None,
        },
        AppCard {
            name: tr.get("cards.game_room.name"),
            description: tr.get("cards.game_room.description"),
            href: "/game-room",
            has_status: false,
            status: String::new(),
            background: "game-room-card-back.jpg",
            service_scope: Some("service:game_room"),
            maintenance: None,
        },
        AppCard {
            name: tr.get("cards.initiative.name"),
            description: tr.get("cards.initiative.description"),
            href: "#",
            has_status: true,
            status: coming_soon.clone(),
            background: "initiative-card-back.png",
            service_scope: Some("service:initiative"),
            maintenance: None,
        },
        AppCard {
            name: tr.get("cards.systems.name"),
            description: tr.get("cards.systems.description"),
            href: "#",
            has_status: true,
            status: coming_soon.clone(),
            background: "systems-card-back.jpg",
            service_scope: Some("service:systems"),
            maintenance: None,
        },
        AppCard {
            name: tr.get("cards.profile.name"),
            description: tr.get("cards.profile.description"),
            href: "#",
            has_status: false,
            status: String::new(),
            background: "profile-card-back.jpg",
            service_scope: Some("service:users"),
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
    shared_url: String,
    apps: Vec<AppCard>,
    version: String,
    build_date: String,
    build_hash: String,
    banners: Vec<Banner>,
    /// `Some(display_name)` when the shared session (see `session_client`) resolves to a
    /// logged-in visitor; `None` otherwise. `auth-web` is the only writer of this session -
    /// this app only ever reads it.
    current_user_name: Option<String>,
    /// Shown as a smaller, muted subtitle line under the name in the avatar menu. `None` when
    /// logged out or the session has no email (same source as `avatar_gravatar_url`).
    current_user_email: Option<String>,
    /// First character of `current_user_name`, uppercased - the avatar trigger's label.
    /// Precomputed here rather than in the template (matching `build_hash`'s existing
    /// precedent above) since Askama's expression syntax doesn't reliably support chained
    /// method calls. Empty when logged out; unused in that branch of the template.
    avatar_initial: String,
    /// Gravatar image URL derived from the session's email (`d=404` so a visitor with no
    /// Gravatar gets a real 404 rather than Gravatar's generic mystery-person image) - the
    /// template's `onerror` falls back to the `avatar_initial` letter circle on load failure.
    /// `None` when logged out or the session has no email.
    avatar_gravatar_url: Option<String>,
    /// `true` when the session's `roles` (verified by `users-api`, see `session_client`)
    /// includes `admin` - gates the avatar menu's "Admin" item, mirroring `admin-web`'s own
    /// `AuthRequiredMiddleware` role check.
    is_admin: bool,
    login_url: String,
    logout_url: String,
    admin_url: String,
    user_settings_url: String,
    /// Per-request translator - the template's source for every user-facing string.
    tr: Tr,
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

/// Builds a Gravatar image URL from a session email, or `None` if no email is present.
/// `d=404` makes Gravatar return a real 404 for an email with no registered image, instead of
/// its generic mystery-person placeholder - the template's `onerror` handler catches that and
/// falls back to the initial-letter circle.
fn gravatar_url(email: Option<&str>) -> Option<String> {
    let email = email?;
    let hash = format!("{:x}", md5::compute(email.trim().to_lowercase().as_bytes()));
    Some(format!("https://www.gravatar.com/avatar/{hash}?s=64&d=404"))
}

/// Maps auth-web's closed set of `login_error` reason codes (`AuthController.swift`'s
/// `LoginErrorReason`) to a `login_error.*` locale key. Never echoes the raw query value - an
/// unrecognized reason (including the old bare `1` flag this replaces) falls back to the
/// generic key, so there's nothing here an attacker could use to inject arbitrary text into
/// the page.
fn login_error_message_key(reason: &str) -> &'static str {
    match reason {
        "denied" => "login_error.denied",
        "expired" => "login_error.expired",
        "unavailable" => "login_error.unavailable",
        "forbidden" => "login_error.forbidden",
        _ => "login_error.generic",
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(index))
}

async fn index(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IndexQuery>,
    jar: CookieJar,
    headers: HeaderMap,
) -> LandingTemplate {
    let accept_language = headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok());
    let tr = Tr::resolve(&jar, accept_language);

    let mut banners = state
        .admin_client
        .fetch_banners(&["platform", "service:main"])
        .await;

    let mut apps = apps(&tr);
    let mut maintenance_scopes = vec!["platform"];
    maintenance_scopes.extend(apps.iter().filter_map(|app| app.service_scope));
    let maintenance_records = state
        .admin_client
        .fetch_maintenance_modes(&maintenance_scopes)
        .await;
    apply_maintenance(&mut apps, &maintenance_records);

    // auth-web (the suite's sole Auth0 callback handler) redirects back here with
    // ?login_error=<reason> on any login failure - render it via the same banner markup
    // admin-api's banners use, rather than a bespoke alert element, so it gets the existing
    // styling/theming for free. Synthesized client-side, not from admin-api, so it's never
    // cached or shared with other visitors. `reason` is one of a small closed set auth-web
    // defines (AuthController.swift's LoginErrorReason) - never raw Auth0/users-api error
    // text, so there's nothing here that could leak internal detail even for an unrecognized
    // value.
    if let Some(reason) = query.login_error.as_deref() {
        banners.insert(
            0,
            Banner {
                severity: "critical".to_string(),
                message: tr.get(login_error_message_key(reason)),
            },
        );
    }

    let current_user = match jar.get(SESSION_COOKIE_NAME) {
        Some(cookie) => state.session_client.current_user(cookie.value()).await,
        None => None,
    };
    let is_admin = current_user
        .as_ref()
        .is_some_and(|user| user.roles.iter().any(|role| role == "admin"));
    let avatar_initial = current_user
        .as_ref()
        .and_then(|user| user.name.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    let avatar_gravatar_url = current_user
        .as_ref()
        .and_then(|user| gravatar_url(user.email.as_deref()));
    let current_user_email = current_user.as_ref().and_then(|user| user.email.clone());
    let current_user_name = current_user.map(|user| user.name);

    // No INGRESS_BASE_PATH prefix needed, unlike catalog-web/admin-web: auth-web sits at
    // /auth on this same host, and main-web itself serves the host's root - see
    // design.md's "auth-web is the sole owner of the Authorization Code exchange" decision.
    LandingTemplate {
        shared_url: state.config.shared_url.clone(),
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
        current_user_email,
        avatar_initial,
        avatar_gravatar_url,
        is_admin,
        login_url: "/auth/login?return_to=/".to_string(),
        logout_url: "/auth/logout".to_string(),
        // Fixed paths on the shared `dev.sweetrpg.com` host, matching `/catalog`'s convention -
        // see design.md's "Profile links to a fixed, currently-unbuilt path" decision.
        admin_url: "/admin".to_string(),
        user_settings_url: "/users/profile".to_string(),
        tr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template(current_user_name: Option<String>, is_admin: bool) -> LandingTemplate {
        LandingTemplate {
            shared_url: "http://localhost:8081".to_string(),
            apps: apps(&Tr::english()),
            version: "dev".to_string(),
            build_date: "unset".to_string(),
            build_hash: "unset".to_string(),
            banners: Vec::new(),
            avatar_initial: current_user_name
                .as_deref()
                .and_then(|n| n.chars().next())
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default(),
            current_user_name,
            current_user_email: None,
            avatar_gravatar_url: None,
            is_admin,
            login_url: "/auth/login?return_to=/".to_string(),
            logout_url: "/auth/logout".to_string(),
            admin_url: "/admin".to_string(),
            user_settings_url: "/users/profile".to_string(),
            tr: Tr::english(),
        }
    }

    #[test]
    fn logged_out_shows_avatar_menu_with_mystery_man_and_log_in_item() {
        let html = template(None, false).render().expect("template renders");
        assert!(html.contains(r#"href="/auth/login?return_to=/""#));
        assert!(html.contains("Log in"));
        assert!(!html.contains("Log out"));
        assert!(html.contains("avatar-menu-trigger"));
        assert!(html.contains("mystery-man.svg"));
        assert!(html.contains("avatar-menu-theme-row"));
    }

    #[test]
    fn app_switcher_renders_with_four_destinations_and_no_admin_link() {
        let html = template(None, false).render().expect("template renders");
        assert!(html.contains("app-switcher-trigger"));
        assert!(html.contains(r#"href="/">Main"#));
        assert!(html.contains(r#"href="/catalog">Catalog"#));
        assert!(html.contains(r#"href="/game-room">Game Room"#));
        assert!(html.contains(r#"href="/initiative">Initiative"#));
        assert!(!html.contains("app-switcher-item\" href=\"/admin\""));
    }

    #[test]
    fn login_error_query_parses_from_form_encoded_flag() {
        let query: IndexQuery = serde_urlencoded::from_str("login_error=expired").unwrap();
        assert_eq!(query.login_error.as_deref(), Some("expired"));

        let query: IndexQuery = serde_urlencoded::from_str("").unwrap();
        assert_eq!(query.login_error, None);
    }

    #[test]
    fn login_error_message_covers_every_known_reason_distinctly() {
        let known = ["denied", "expired", "unavailable", "forbidden"];
        let messages: Vec<&str> = known.iter().map(|r| login_error_message_key(r)).collect();
        // Every known reason gets its own distinct copy - no two collapse to the same message.
        let mut unique = messages.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), known.len());
    }

    #[test]
    fn login_error_message_falls_back_for_unrecognized_reason() {
        // Covers the legacy bare `1` flag this replaced, and any unrecognized value - never
        // echoes the input back into the message.
        assert_eq!(login_error_message_key("1"), "login_error.generic");
        assert_eq!(
            login_error_message_key("<script>alert(1)</script>"),
            "login_error.generic"
        );
    }

    #[test]
    fn every_login_error_key_resolves_to_english_copy() {
        let tr = Tr::english();
        for reason in [
            "denied",
            "expired",
            "unavailable",
            "forbidden",
            "anything-else",
        ] {
            let message = tr.get(login_error_message_key(reason));
            assert!(!message.is_empty());
            assert_ne!(message, login_error_message_key(reason), "missing key");
        }
    }

    #[test]
    fn login_error_banner_renders_as_critical() {
        let mut tpl = template(None, false);
        tpl.banners.push(Banner {
            severity: "critical".to_string(),
            message: "Login failed. Please try again.".to_string(),
        });
        let html = tpl.render().expect("template renders");
        assert!(html.contains("banner-critical"));
        assert!(html.contains("Login failed. Please try again."));
    }

    #[test]
    fn logged_in_shows_avatar_menu_with_name_and_log_out_form() {
        let html = template(Some("Alice".to_string()), false)
            .render()
            .expect("template renders");
        assert!(html.contains("Alice"));
        assert!(html.contains(r#"action="/auth/logout""#));
        assert!(html.contains("Log out"));
        assert!(!html.contains(">Log in<"));
        assert!(html.contains("avatar-menu-trigger"));
        assert!(html.contains("avatar-menu-item-danger"));
        assert!(!html.contains("mystery-man.svg"));
        assert!(html.contains("avatar-menu-theme-row"));
    }

    #[test]
    fn logged_in_with_email_shows_it_as_a_subtitle() {
        let mut tpl = template(Some("Alice".to_string()), false);
        tpl.current_user_email = Some("alice@example.com".to_string());
        let html = tpl.render().expect("template renders");
        assert!(html.contains("avatar-menu-email"));
        assert!(html.contains("alice@example.com"));
    }

    #[test]
    fn logged_in_without_email_shows_no_subtitle() {
        let html = template(Some("Alice".to_string()), false)
            .render()
            .expect("template renders");
        assert!(!html.contains("avatar-menu-email"));
    }

    #[test]
    fn logged_in_shows_user_settings_but_not_admin_when_not_admin() {
        let html = template(Some("Alice".to_string()), false)
            .render()
            .expect("template renders");
        assert!(html.contains(r#"href="/users/profile""#));
        assert!(!html.contains(r#"href="/admin""#));
    }

    #[test]
    fn admin_session_shows_admin_link() {
        let html = template(Some("Alice".to_string()), true)
            .render()
            .expect("template renders");
        assert!(html.contains(r#"href="/admin""#));
        assert!(html.contains(r#"href="/users/profile""#));
    }

    #[test]
    fn gravatar_url_is_none_without_an_email() {
        assert_eq!(gravatar_url(None), None);
    }

    #[test]
    fn gravatar_url_hashes_a_trimmed_lowercased_email_with_a_404_fallback() {
        let mixed_case = gravatar_url(Some(" Alice@Example.com ")).unwrap();
        let canonical = gravatar_url(Some("alice@example.com")).unwrap();
        assert_eq!(mixed_case, canonical);
        assert!(canonical.starts_with("https://www.gravatar.com/avatar/"));
        assert!(canonical.contains("d=404"));
    }

    #[test]
    fn logged_in_with_email_renders_gravatar_img_with_onerror_fallback() {
        let mut tpl = template(Some("Alice".to_string()), false);
        tpl.avatar_gravatar_url = gravatar_url(Some("alice@example.com"));
        let html = tpl.render().expect("template renders");
        assert!(html.contains("avatar-menu-avatar"));
        assert!(html.contains(
            "onerror=\"this.style.display='none'; this.nextElementSibling.style.display='flex';\""
        ));
        assert!(html.contains("avatar-menu-fallback"));
    }

    #[test]
    fn logged_in_without_email_renders_only_the_fallback_letter() {
        let html = template(Some("Alice".to_string()), false)
            .render()
            .expect("template renders");
        assert!(!html.contains("avatar-menu-avatar"));
        assert!(html.contains("avatar-menu-fallback"));
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
        let mut apps = apps(&Tr::english());
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
        let mut apps = apps(&Tr::english());
        apply_maintenance(&mut apps, &[maintenance_mode("platform", "")]);

        for app in &apps {
            assert_eq!(app.maintenance.is_some(), app.service_scope.is_some());
        }
    }

    #[test]
    fn maintenance_badge_renders_with_tooltip() {
        let mut tpl = template(None, false);
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
