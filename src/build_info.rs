use std::env;
use std::fs;

/// Read from the JSON file the Dockerfile bakes in at build time (same `BUILD_INFO_PATH`
/// convention as `catalog-web`'s `BuildInfo.swift` / the Go services'
/// `docs/service-conventions.md`), not generated at runtime: falls back to placeholder values
/// when the file doesn't exist (local `cargo run`, outside a container).
#[derive(Clone, serde::Deserialize)]
pub struct BuildInfo {
    #[serde(default = "unset")]
    pub number: String,
    #[serde(default = "unset")]
    pub job: String,
    #[serde(default = "unset_sha")]
    pub sha: String,
    #[serde(default = "unset")]
    pub date: String,
    #[serde(default = "dev")]
    pub version: String,
}

fn unset() -> String {
    "unset".to_string()
}

/// 8 hex chars, so it needs no truncation guard where a real short commit SHA would - and
/// reads unmistakably as a placeholder rather than a real hash.
fn unset_sha() -> String {
    "deadbeef".to_string()
}

fn dev() -> String {
    "dev".to_string()
}

impl BuildInfo {
    pub fn load() -> Self {
        let path =
            env::var("BUILD_INFO_PATH").unwrap_or_else(|_| "/app/config/build-info.json".into());
        fs::read_to_string(&path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or(BuildInfo {
                number: unset(),
                job: unset(),
                sha: unset_sha(),
                date: unset(),
                version: dev(),
            })
    }
}
