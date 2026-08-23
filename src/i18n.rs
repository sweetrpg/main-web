use axum_extra::extract::CookieJar;

/// The default/fallback locale. English catalogs are the only ones shipped today;
/// new locales land as `locales/<code>.yml` plus an entry in `SUPPORTED_LOCALES`.
pub const DEFAULT_LOCALE: &str = "en";

/// Cookie set by a locale-override request; wins over `Accept-Language`.
pub const LOCALE_COOKIE_NAME: &str = "locale";

/// Locales this app can actually render. Resolution falls back to
/// [`DEFAULT_LOCALE`] for anything else.
const SUPPORTED_LOCALES: &[&str] = &[DEFAULT_LOCALE];

/// Per-request translator handed to Askama templates. Templates call its methods
/// (`{{ tr.tagline() }}`) because Askama can't invoke the `t!` macro itself.
#[derive(Clone)]
pub struct Tr {
    locale: String,
}

impl Tr {
    pub fn new(locale: impl Into<String>) -> Self {
        Self {
            locale: locale.into(),
        }
    }

    /// English-only translator, for tests and non-request callers.
    pub fn english() -> Self {
        Self::new(DEFAULT_LOCALE)
    }

    fn s(&self, key: &str) -> String {
        rust_i18n::t!(key, locale = self.locale.as_str()).to_string()
    }

    /// Translates an arbitrary key - used for dynamic keys like the per-card
    /// `cards.<id>.name` lookups.
    pub fn get(&self, key: &str) -> String {
        self.s(key)
    }

    /// Resolves the request's locale per the `web-frontend-localization` spec:
    /// cookie override, then `Accept-Language`, then English.
    pub fn resolve(jar: &CookieJar, accept_language: Option<&str>) -> Self {
        if let Some(cookie_locale) = jar.get(LOCALE_COOKIE_NAME).map(|c| c.value()) {
            if SUPPORTED_LOCALES.contains(&cookie_locale) {
                return Self::new(cookie_locale);
            }
        }
        if let Some(header) = accept_language {
            // First tag only - quality values are ordered by convention here, and a
            // full q-value parser isn't warranted while one locale is supported.
            if let Some(tag) = header.split(',').next().and_then(|t| t.split(';').next()) {
                let tag = tag.trim();
                let base = tag.split('-').next().unwrap_or(tag);
                if SUPPORTED_LOCALES.contains(&base) {
                    return Self::new(base);
                }
            }
        }
        Self::english()
    }

    pub fn nav_main(&self) -> String {
        self.s("nav.main")
    }
    pub fn nav_catalog(&self) -> String {
        self.s("nav.catalog")
    }
    pub fn nav_game_systems(&self) -> String {
        self.s("nav.game_systems")
    }
    pub fn nav_shelf(&self) -> String {
        self.s("nav.shelf")
    }
    pub fn nav_initiative(&self) -> String {
        self.s("nav.initiative")
    }
    pub fn menu_switch_app(&self) -> String {
        self.s("menu.switch_app")
    }
    pub fn menu_theme(&self) -> String {
        self.s("menu.theme")
    }
    pub fn menu_theme_light(&self) -> String {
        self.s("menu.theme_light")
    }
    pub fn menu_theme_dark(&self) -> String {
        self.s("menu.theme_dark")
    }
    pub fn menu_theme_system(&self) -> String {
        self.s("menu.theme_system")
    }
    pub fn menu_user_settings(&self) -> String {
        self.s("menu.user_settings")
    }
    pub fn menu_administration(&self) -> String {
        self.s("menu.administration")
    }
    pub fn menu_log_out(&self) -> String {
        self.s("menu.log_out")
    }
    pub fn menu_log_in(&self) -> String {
        self.s("menu.log_in")
    }
    pub fn landing_tagline(&self) -> String {
        self.s("landing.tagline")
    }
    pub fn cards_maintenance(&self) -> String {
        self.s("cards.maintenance")
    }
    pub fn cards_coming_soon(&self) -> String {
        self.s("cards.coming_soon")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_override_wins_over_accept_language() {
        let mut jar = CookieJar::new();
        jar = jar.add(("locale", "en"));
        let tr = Tr::resolve(&jar, Some("zz"));
        assert_eq!(tr.locale, "en");
    }

    #[test]
    fn unsupported_cookie_falls_through_to_accept_language() {
        let mut jar = CookieJar::new();
        jar = jar.add((LOCALE_COOKIE_NAME, "zz"));
        let tr = Tr::resolve(&jar, Some("en"));
        assert_eq!(tr.locale, "en");
    }

    #[test]
    fn accept_language_region_tag_matches_base_locale() {
        let tr = Tr::resolve(&CookieJar::new(), Some("en-GB,en;q=0.9"));
        assert_eq!(tr.locale, "en");
    }

    #[test]
    fn unsupported_accept_language_and_no_cookie_falls_back_to_english() {
        let tr = Tr::resolve(&CookieJar::new(), Some("fr-CA,fr;q=0.9"));
        assert_eq!(tr.locale, DEFAULT_LOCALE);
    }

    #[test]
    fn missing_everything_falls_back_to_english() {
        let tr = Tr::resolve(&CookieJar::new(), None);
        assert_eq!(tr.locale, DEFAULT_LOCALE);
    }

    #[test]
    fn english_translator_resolves_a_known_key() {
        assert!(!Tr::english().landing_tagline().is_empty());
    }
}
