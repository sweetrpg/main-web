use axum::http::HeaderMap;
use axum_extra::extract::CookieJar;
use chrono::Datelike;

use crate::constants::{PRIVACY_PATH, SUPPORT_PATH, TERMS_PATH};
use crate::i18n::Tr;
use crate::session_client::{SessionUser, SESSION_COOKIE_NAME};
use crate::AppState;

/// Values the shared footer and asset partials (`templates/partials/`) need on every page.
pub struct Chrome {
    pub shared_url: String,
    pub version: String,
    pub build_date: String,
    pub build_hash: String,
    pub year: i32,
    pub terms_path: &'static str,
    pub privacy_path: &'static str,
    pub support_path: &'static str,
    /// Wrapper class for the footer: the landing page pins it to the viewport and fades it
    /// (`landing-footer`), every other page uses the shared `footer`.
    pub footer_class: &'static str,
    pub footer_line_class: &'static str,
    pub tr: Tr,
}

impl Chrome {
    pub fn new(state: &AppState, tr: Tr) -> Self {
        Self {
            shared_url: state.config.shared_url.clone(),
            version: state.build_info.version.clone(),
            build_date: state.build_info.date.clone(),
            build_hash: state.build_info.short_sha(),
            year: chrono::Utc::now().year(),
            terms_path: TERMS_PATH,
            privacy_path: PRIVACY_PATH,
            support_path: SUPPORT_PATH,
            footer_class: "footer",
            footer_line_class: "",
            tr,
        }
    }

    /// The landing page's fixed, fading footer variant.
    pub fn landing(mut self) -> Self {
        self.footer_class = "landing-footer";
        self.footer_line_class = "landing-footer-line";
        self
    }
}

/// Signed-in or signed-out state for the shared header (`partials/header.html`).
pub struct HeaderMenu {
    /// `Some(display_name)` when the shared session (see `session_client`) resolves to a
    /// logged-in visitor; `None` otherwise. `auth-web` is the only writer of this session -
    /// this app only ever reads it.
    pub current_user_name: Option<String>,
    /// Shown as a smaller, muted subtitle line under the name in the avatar menu. `None` when
    /// logged out or the session has no email (same source as `avatar_gravatar_url`).
    pub current_user_email: Option<String>,
    /// First character of `current_user_name`, uppercased - the avatar trigger's label.
    /// Precomputed because Askama's expression syntax doesn't reliably support chained method
    /// calls. Empty when logged out; unused in that branch of the template.
    pub avatar_initial: String,
    /// Gravatar image URL derived from the session's email (`d=404` so a visitor with no
    /// Gravatar gets a real 404 rather than Gravatar's generic mystery-person image) - the
    /// template's `onerror` falls back to the `avatar_initial` letter circle on load failure.
    pub avatar_gravatar_url: Option<String>,
    /// `true` when the session's `roles` (verified by `users-api`) include `admin` - gates the
    /// avatar menu's administration item, mirroring `admin-web`'s own role check.
    pub is_admin: bool,
    pub login_url: String,
    pub logout_url: String,
    pub admin_url: String,
    pub user_settings_url: String,
    /// Public `admin-api` base URL for the feedback widget's `data-api-url` attribute.
    pub feedback_api_url: String,
}

impl HeaderMenu {
    /// Reads the shared session the way the landing page always has; `return_to` is where
    /// login sends the visitor back to.
    pub async fn resolve(state: &AppState, jar: &CookieJar, return_to: &str) -> Self {
        let user = match jar.get(SESSION_COOKIE_NAME) {
            Some(cookie) => state.session_client.current_user(cookie.value()).await,
            None => None,
        };
        Self::new(user, &state.config.feedback_api_url, return_to)
    }

    pub fn new(user: Option<SessionUser>, feedback_api_url: &str, return_to: &str) -> Self {
        let is_admin = user
            .as_ref()
            .is_some_and(|u| u.roles.iter().any(|role| role == "admin"));
        let avatar_initial = user
            .as_ref()
            .and_then(|u| u.name.chars().next())
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default();
        let avatar_gravatar_url = user.as_ref().and_then(|u| gravatar_url(u.email.as_deref()));
        let current_user_email = user.as_ref().and_then(|u| u.email.clone());
        Self {
            current_user_name: user.map(|u| u.name),
            current_user_email,
            avatar_initial,
            avatar_gravatar_url,
            is_admin,
            // No INGRESS_BASE_PATH prefix: auth-web sits at /auth on this same host and
            // main-web serves the host's root.
            login_url: format!("/auth/login?return_to={return_to}"),
            logout_url: "/auth/logout".to_string(),
            admin_url: "/admin".to_string(),
            user_settings_url: "/users/profile".to_string(),
            feedback_api_url: feedback_api_url.to_string(),
        }
    }
}

/// Resolves the request's translator from the locale cookie and `Accept-Language`.
pub fn translator(jar: &CookieJar, headers: &HeaderMap) -> Tr {
    let accept_language = headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok());
    Tr::resolve(jar, accept_language)
}

/// Builds a Gravatar image URL from a session email, or `None` if no email is present.
/// `d=404` makes Gravatar return a real 404 for an email with no registered image, instead of
/// its generic mystery-person placeholder - the template's `onerror` handler catches that and
/// falls back to the initial-letter circle.
pub fn gravatar_url(email: Option<&str>) -> Option<String> {
    let email = email?;
    let hash = format!("{:x}", md5::compute(email.trim().to_lowercase().as_bytes()));
    Some(format!("https://www.gravatar.com/avatar/{hash}?s=64&d=404"))
}
