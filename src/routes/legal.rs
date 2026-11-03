use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::CookieJar;

use crate::build_info::BuildInfo;
use crate::config::{Config, SupportContact};
use crate::constants::{PRIVACY_PATH, SUPPORT_PATH, TERMS_PATH};
use crate::i18n::Tr;
use crate::legal_doc::{LegalDoc, LegalStatus};
use crate::AppState;

/// Values the shared page layout (`base_page.html`) needs on every public page.
struct Chrome {
    shared_url: String,
    version: String,
    build_date: String,
    build_hash: String,
    terms_path: &'static str,
    privacy_path: &'static str,
    support_path: &'static str,
    tr: Tr,
}

impl Chrome {
    fn new(config: &Config, build_info: &BuildInfo, tr: Tr) -> Self {
        Self {
            shared_url: config.shared_url.clone(),
            version: build_info.version.clone(),
            build_date: build_info.date.clone(),
            build_hash: build_info.short_sha(),
            terms_path: TERMS_PATH,
            privacy_path: PRIVACY_PATH,
            support_path: SUPPORT_PATH,
            tr,
        }
    }
}

#[derive(Template)]
#[template(path = "document.html")]
struct DocumentTemplate {
    chrome: Chrome,
    title: String,
    is_draft: bool,
    version_label: String,
    effective_date: String,
    body_html: String,
}

impl DocumentTemplate {
    fn new(chrome: Chrome, title: String, doc: &LegalDoc) -> Self {
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
            title,
        }
    }
}

#[derive(Template)]
#[template(path = "support.html")]
struct SupportTemplate {
    chrome: Chrome,
    contact_display: String,
    contact_href: String,
}

impl SupportTemplate {
    fn new(chrome: Chrome, contact: &SupportContact) -> Self {
        Self {
            chrome,
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

fn chrome(state: &AppState, jar: &CookieJar, headers: &HeaderMap) -> Chrome {
    let accept_language = headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok());
    let tr = Tr::resolve(jar, accept_language);
    Chrome::new(&state.config, &state.build_info, tr)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(TERMS_PATH, get(terms))
        .route(PRIVACY_PATH, get(privacy))
        .route(SUPPORT_PATH, get(support))
}

async fn terms(State(state): State<Arc<AppState>>, jar: CookieJar, headers: HeaderMap) -> Response {
    let chrome = chrome(&state, &jar, &headers);
    let title = chrome.tr.terms_title();
    render(DocumentTemplate::new(chrome, title, &state.terms), "terms")
}

async fn privacy(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    let chrome = chrome(&state, &jar, &headers);
    let title = chrome.tr.privacy_title();
    render(
        DocumentTemplate::new(chrome, title, &state.privacy),
        "privacy",
    )
}

async fn support(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    let page = SupportTemplate::new(
        chrome(&state, &jar, &headers),
        &state.config.support_contact,
    );
    render(page, "support")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{PRIVACY_SOURCE, TERMS_SOURCE};

    fn test_chrome() -> Chrome {
        Chrome {
            shared_url: "http://localhost:8081".to_string(),
            version: "dev".to_string(),
            build_date: "unset".to_string(),
            build_hash: "deadbeef".to_string(),
            terms_path: TERMS_PATH,
            privacy_path: PRIVACY_PATH,
            support_path: SUPPORT_PATH,
            tr: Tr::english(),
        }
    }

    fn terms_html(header_status: &str, effective_date: &str) -> String {
        let source = format!(
            "---\nversion: \"2.3\"\nstatus: {header_status}\neffective_date: \"{effective_date}\"\nmaterial: false\n---\n# Terms\n\nBody text.\n"
        );
        let doc = LegalDoc::parse(&source).expect("valid terms");
        DocumentTemplate::new(test_chrome(), "Terms".to_string(), &doc)
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
            let html = DocumentTemplate::new(test_chrome(), "Doc".to_string(), &doc)
                .render()
                .expect("template renders");
            assert!(html.contains(&doc.header.version));
        }
    }

    #[test]
    fn support_page_links_to_terms_and_the_contact() {
        let contact = SupportContact::parse("help@example.com").unwrap();
        let html = SupportTemplate::new(test_chrome(), &contact)
            .render()
            .expect("template renders");
        assert!(html.contains(r#"href="/terms""#));
        assert!(html.contains(r#"href="mailto:help@example.com""#));
    }

    #[test]
    fn every_page_footer_links_terms_and_support() {
        let contact = SupportContact::parse("help@example.com").unwrap();
        let html = SupportTemplate::new(test_chrome(), &contact)
            .render()
            .expect("template renders");
        assert!(html.contains(r#"href="/support""#));
        assert!(html.contains(r#"href="/privacy""#));
    }
}
