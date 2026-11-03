use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::CookieJar;

use crate::config::SupportContact;
use crate::constants::{PRIVACY_PATH, SUPPORT_PATH, TERMS_PATH};
use crate::legal_doc::{LegalDoc, LegalStatus};
use crate::page::{translator, Chrome, HeaderMenu};
use crate::AppState;

#[derive(Template)]
#[template(path = "document.html")]
struct DocumentTemplate {
    chrome: Chrome,
    menu: HeaderMenu,
    title: String,
    is_draft: bool,
    version_label: String,
    effective_date: String,
    body_html: String,
}

impl DocumentTemplate {
    fn new(chrome: Chrome, menu: HeaderMenu, title: String, doc: &LegalDoc) -> Self {
        let header = &doc.header;
        let effective_date = if header.effective_date.is_empty() {
            chrome.tr.terms_effective_date_unset()
        } else {
            header.effective_date.clone()
        };
        Self {
            is_draft: header.status == LegalStatus::Draft,
            version_label: header.version.clone(),
            effective_date,
            body_html: doc.html.clone(),
            chrome,
            menu,
            title,
        }
    }
}

#[derive(Template)]
#[template(path = "support.html")]
struct SupportTemplate {
    chrome: Chrome,
    menu: HeaderMenu,
    contact_display: String,
    contact_href: String,
}

impl SupportTemplate {
    fn new(chrome: Chrome, menu: HeaderMenu, contact: &SupportContact) -> Self {
        Self {
            chrome,
            menu,
            contact_display: contact.display.clone(),
            contact_href: contact.href.clone(),
        }
    }
}

fn render(template: impl Template, page: &str) -> Response {
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(err) => {
            tracing::error!(error = %err, page, "failed to render page template");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(TERMS_PATH, get(terms))
        .route(PRIVACY_PATH, get(privacy))
        .route(SUPPORT_PATH, get(support))
}

/// Shared header/footer context for a public page; `path` is where login returns to.
async fn page_context(
    state: &AppState,
    jar: &CookieJar,
    headers: &HeaderMap,
    path: &str,
) -> (Chrome, HeaderMenu) {
    let chrome = Chrome::new(state, translator(jar, headers));
    let menu = HeaderMenu::resolve(state, jar, path).await;
    (chrome, menu)
}

async fn terms(State(state): State<Arc<AppState>>, jar: CookieJar, headers: HeaderMap) -> Response {
    let (chrome, menu) = page_context(&state, &jar, &headers, TERMS_PATH).await;
    let title = chrome.tr.terms_title();
    let page = DocumentTemplate::new(chrome, menu, title, &state.terms);
    render(page, "terms")
}

async fn privacy(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    let (chrome, menu) = page_context(&state, &jar, &headers, PRIVACY_PATH).await;
    let title = chrome.tr.privacy_title();
    let page = DocumentTemplate::new(chrome, menu, title, &state.privacy);
    render(page, "privacy")
}

async fn support(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    let (chrome, menu) = page_context(&state, &jar, &headers, SUPPORT_PATH).await;
    let page = SupportTemplate::new(chrome, menu, &state.config.support_contact);
    render(page, "support")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{PRIVACY_SOURCE, TERMS_SOURCE};
    use crate::i18n::Tr;
    use crate::session_client::SessionUser;

    fn test_chrome() -> Chrome {
        Chrome {
            shared_url: "http://localhost:8081".to_string(),
            version: "1.2.3".to_string(),
            build_date: "2026-11-03".to_string(),
            build_hash: "deadbeef".to_string(),
            year: 2026,
            terms_path: TERMS_PATH,
            privacy_path: PRIVACY_PATH,
            support_path: SUPPORT_PATH,
            footer_class: "footer",
            footer_line_class: "",
            tr: Tr::english(),
        }
    }

    fn signed_out() -> HeaderMenu {
        HeaderMenu::new(None, "/api/0/admin/feedback", TERMS_PATH)
    }

    fn signed_in() -> HeaderMenu {
        let user = SessionUser {
            sub: "sub".to_string(),
            name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            roles: Vec::new(),
            expiry: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        HeaderMenu::new(Some(user), "/api/0/admin/feedback", TERMS_PATH)
    }

    fn support_html(menu: HeaderMenu) -> String {
        let contact = SupportContact::parse("help@example.com").unwrap();
        SupportTemplate::new(test_chrome(), menu, &contact)
            .render()
            .expect("template renders")
    }

    fn terms_html(header_status: &str, effective_date: &str) -> String {
        let source = format!(
            "---\nversion: \"2.3\"\nstatus: {header_status}\neffective_date: \"{effective_date}\"\nmaterial: false\n---\n# Terms\n\nBody text.\n"
        );
        let doc = LegalDoc::parse(&source).expect("valid terms");
        DocumentTemplate::new(test_chrome(), signed_out(), "Terms".to_string(), &doc)
            .render()
            .expect("template renders")
    }

    #[test]
    fn draft_terms_show_the_not_in_force_notice() {
        let html = terms_html("draft", "");
        assert!(html.contains("This is a draft and is not in force"));
    }

    #[test]
    fn in_force_terms_hide_the_draft_notice() {
        let html = terms_html("in_force", "2026-11-01");
        assert!(!html.contains("not in force"));
    }

    #[test]
    fn terms_show_version_and_effective_date() {
        let html = terms_html("in_force", "2026-11-01");
        assert!(html.contains("2.3"));
        assert!(html.contains("2026-11-01"));
    }

    #[test]
    fn terms_without_an_effective_date_say_so() {
        let html = terms_html("draft", "");
        assert!(html.contains("Not yet set"));
    }

    #[test]
    fn terms_render_the_markdown_body_unescaped() {
        let html = terms_html("draft", "");
        assert!(html.contains("<h1>Terms</h1>"));
        assert!(html.contains("<p>Body text.</p>"));
    }

    #[test]
    fn embedded_documents_render_with_their_version() {
        for source in [TERMS_SOURCE, PRIVACY_SOURCE] {
            let doc = LegalDoc::parse(source).expect("embedded document parses");
            let html = DocumentTemplate::new(test_chrome(), signed_out(), "Doc".to_string(), &doc)
                .render()
                .expect("template renders");
            assert!(html.contains(&doc.header.version));
        }
    }

    #[test]
    fn support_page_links_to_terms_and_the_contact() {
        let html = support_html(signed_out());
        assert!(html.contains(r#"href="/terms""#));
        assert!(html.contains(r#"href="mailto:help@example.com""#));
    }

    #[test]
    fn signed_out_pages_show_app_switcher_and_log_in_menu() {
        let html = support_html(signed_out());
        assert!(html.contains("app-switcher-trigger"));
        assert!(html.contains(r#"href="/catalog">Catalog"#));
        assert!(html.contains("avatar-menu-trigger"));
        assert!(html.contains("mystery-man.svg"));
        assert!(html.contains(r#"href="/auth/login?return_to=/terms""#));
        assert!(html.contains(r#"id="feedback-trigger""#));
    }

    #[test]
    fn signed_in_pages_show_identity_and_log_out() {
        let html = support_html(signed_in());
        assert!(html.contains("app-switcher-trigger"));
        assert!(html.contains("Alice"));
        assert!(html.contains("alice@example.com"));
        assert!(html.contains(r#"action="/auth/logout""#));
        assert!(html.contains(r#"href="/users/profile""#));
        assert!(!html.contains(">Log in<"));
    }

    #[test]
    fn document_pages_render_the_same_header() {
        let html = terms_html("in_force", "2026-11-01");
        assert!(html.contains("app-switcher-trigger"));
        assert!(html.contains("avatar-menu-trigger"));
    }

    #[test]
    fn footer_has_three_lines_in_order_with_root_legal_links() {
        let html = support_html(signed_out());
        let copyright = html
            .find("&copy; 2026 Pilgrimage Software")
            .expect("line 1");
        let links = html
            .find(r#"href="/terms">Terms of Service</a>"#)
            .expect("line 2");
        let build = html
            .find("v1.2.3 &middot; built 2026-11-03 / deadbeef")
            .expect("line 3");
        assert!(copyright < links && links < build);
        assert!(html.contains(
            r#"href="/privacy">Privacy Policy</a> &middot; <a href="/support">Support</a>"#
        ));
        assert_eq!(html.matches("footer-line").count(), 3);
    }
}
