use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::admin_client::{AppCardStatus, Banner, MaintenanceMode};
use crate::i18n::Tr;
use crate::page::{translator, Chrome, HeaderMenu};
use crate::AppState;

/// The suite's app list is a compile-time data structure, not a config file — see
/// `design.md`'s "Decisions" section in the `main-web-rust-rewrite` OpenSpec change for why a
/// YAML/JSON config was rejected (YAGNI: adding a new app already requires a code change and
/// redeploy to update its `href`).
struct AppCard {
    name: String,
    description: String,
    href: &'static str,
    /// Populated after `apps()` returns, from the active app-card-status records - not
    /// compile-time data. Paired with `status` to render the status pill.
    has_status: bool,
    /// Populated after `apps()` returns by `apply_app_card_status()`, containing the label text
    /// from the active record. Ignored if `has_status` is `false`.
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
            has_status: false,
            status: String::new(),
            background: "initiative-card-back.png",
            service_scope: Some("service:initiative"),
            maintenance: None,
        },
        AppCard {
            name: tr.get("cards.systems.name"),
            description: tr.get("cards.systems.description"),
            href: "/game-systems",
            has_status: false,
            status: String::new(),
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

/// Resolves each card's `has_status` and `status` fields from the active app-card-status
/// records. A card with an active record gets `has_status: true` with the record's label text;
/// a card without a record keeps `has_status: false` and `status: String::new()`.
fn apply_app_card_status(apps: &mut [AppCard], records: &[AppCardStatus]) {
    for app in apps.iter_mut() {
        let Some(scope) = app.service_scope else {
            continue;
        };
        let service_value = scope.trim_start_matches("service:");
        if let Some(record) = records
            .iter()
            .find(|r| r.scope_type == "service" && r.scope_value == service_value)
        {
            app.has_status = true;
            app.status = record.label.clone();
        }
    }
}

#[derive(Template)]
#[template(path = "main.html")]
struct LandingTemplate {
    apps: Vec<AppCard>,
    banners: Vec<Banner>,
    chrome: Chrome,
    menu: HeaderMenu,
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
    let tr = translator(&jar, &headers);

    let mut banners = state
        .admin_client
        .fetch_banners(&["platform", "service:main"])
        .await;

    let mut apps = apps(&tr);
    let mut maintenance_scopes = vec!["platform"];
    let app_card_scopes: Vec<&str> = apps.iter().filter_map(|app| app.service_scope).collect();
    maintenance_scopes.extend(app_card_scopes.iter().cloned());
    let maintenance_records = state
        .admin_client
        .fetch_maintenance_modes(&maintenance_scopes)
        .await;
    apply_maintenance(&mut apps, &maintenance_records);

    let app_card_statuses = state
        .admin_client
        .fetch_app_card_statuses(&app_card_scopes)
        .await;
    apply_app_card_status(&mut apps, &app_card_statuses);

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

    let menu = HeaderMenu::resolve(&state, &jar, "/").await;
    let chrome = Chrome::new(&state, tr).landing();

    LandingTemplate {
        apps,
        banners,
        chrome,
        menu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::gravatar_url;
    use crate::session_client::SessionUser;

    fn test_chrome() -> Chrome {
        Chrome {
            shared_url: "http://localhost:8081".to_string(),
            version: "dev".to_string(),
            build_date: "unset".to_string(),
            build_hash: "unset".to_string(),
            year: 2026,
            terms_path: "/terms",
            privacy_path: "/privacy",
            support_path: "/support",
            footer_class: "landing-footer",
            footer_line_class: "landing-footer-line",
            tr: Tr::english(),
        }
    }

    fn template(current_user_name: Option<String>, is_admin: bool) -> LandingTemplate {
        let user = current_user_name.map(|name| SessionUser {
            sub: "sub".to_string(),
            name,
            email: None,
            roles: if is_admin {
                vec!["admin".to_string()]
            } else {
                vec![]
            },
            expiry: chrono::Utc::now() + chrono::Duration::hours(1),
        });
        LandingTemplate {
            apps: apps(&Tr::english()),
            banners: Vec::new(),
            chrome: test_chrome(),
            menu: HeaderMenu::new(user, "/api/0/admin/feedback", "/"),
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
    fn systems_card_links_to_game_systems_web() {
        // game-systems-web now serves dev.sweetrpg.com/game-systems - the Systems card is no
        // longer a dead "#" link. See platform's game-systems-web OpenSpec change.
        let apps = apps(&Tr::english());
        let systems = apps
            .iter()
            .find(|c| c.name == Tr::english().get("cards.systems.name"))
            .expect("Systems card present");
        assert_eq!(systems.href, "/game-systems");
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
        tpl.menu.current_user_email = Some("alice@example.com".to_string());
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
        tpl.menu.avatar_gravatar_url = gravatar_url(Some("alice@example.com"));
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

    fn app_card_status(scope_value: &str, label: &str) -> AppCardStatus {
        AppCardStatus {
            scope_type: "service".to_string(),
            scope_value: scope_value.to_string(),
            label: label.to_string(),
        }
    }

    #[test]
    fn service_scoped_app_card_status_affects_only_that_card() {
        let mut apps = apps(&Tr::english());
        apply_app_card_status(&mut apps, &[app_card_status("catalog", "Beta")]);

        let catalog = apps.iter().find(|a| a.name == "Catalogue").unwrap();
        assert!(catalog.has_status);
        assert_eq!(catalog.status, "Beta");
        let others_unaffected = apps
            .iter()
            .filter(|a| a.name != "Catalogue")
            .all(|a| !a.has_status);
        assert!(others_unaffected);
    }

    #[test]
    fn app_card_status_pill_renders_when_active() {
        let mut tpl = template(None, false);
        tpl.apps[0].has_status = true;
        tpl.apps[0].status = "Beta".to_string();
        let html = tpl.render().expect("template renders");
        assert!(html.contains("tag-outline"));
        assert!(html.contains("Beta"));
    }
}
