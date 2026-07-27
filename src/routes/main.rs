use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;

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
struct HubTemplate {
    shared_assets_url: String,
    apps: Vec<AppCard>,
    version: String,
    build_date: String,
    build_hash: String,
}

impl IntoResponse for HubTemplate {
    fn into_response(self) -> Response {
        match self.render() {
            Ok(body) => Html(body).into_response(),
            Err(err) => {
                tracing::error!(error = %err, "failed to render hub template");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(index))
}

async fn index(State(state): State<Arc<AppState>>) -> HubTemplate {
    HubTemplate {
        shared_assets_url: state.config.shared_assets_url.clone(),
        apps: apps(),
        version: state.build_info.version.clone(),
        build_date: state.build_info.date.clone(),
        build_hash: state.build_info.sha.clone(),
    }
}
