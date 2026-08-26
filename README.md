# Main web

[![CI](https://github.com/sweetrpg/main-web/actions/workflows/ci.yaml/badge.svg)](https://github.com/sweetrpg/main-web/actions/workflows/ci.yaml)
[![Coverage](https://img.shields.io/endpoint?url=https://sweetrpg.github.io/main-web/coverage-badge.json)](https://sweetrpg.github.io/main-web/)
[![License](https://img.shields.io/github/license/sweetrpg/main-web.svg)](https://img.shields.io/github/license/sweetrpg/main-web.svg)
[![Issues](https://img.shields.io/github/issues/sweetrpg/main-web.svg)](https://img.shields.io/github/issues/sweetrpg/main-web.svg)
[![PRs](https://img.shields.io/github/issues-pr/sweetrpg/main-web.svg)](https://img.shields.io/github/issues-pr/sweetrpg/main-web.svg)
[![Deployment](https://argocd.dev.pilgrimagesoftware.com/api/badge?name=sweetrpg-main-web&revision=true&showAppName=true&namespace=sweetrpg-system)](https://argocd.dev.pilgrimagesoftware.com/applications/sweetrpg-main-web)

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
[![Built with love](https://ForTheBadge.com/images/badges/built-with-love.svg)](https://ForTheBadge.com/images/badges/built-with-love.svg)

The landing page for the SweetRPG suite, serving `dev.sweetrpg.com/` (and `sweetrpg.com/` in
production): suite branding, a light/dark/system theme toggle, and a card grid linking out to
each app in the suite (`Catalogue` live, `Game Room`/`Systems`/`Profile`/`Initiative!` marked "Coming
soon" until those frontends exist).

Built with [Axum](https://github.com/tokio-rs/axum) and [Askama](https://github.com/askama-rs/askama)
- the org's first Rust web service (as opposed to the existing Rust library crates like
`common.rs`/`model-core.rs`) and the reference implementation for Rust web-service conventions
on this platform. See `AGENTS.md` for details, and `sweetrpg/platform`'s
`docs/rust-service-conventions.md` for the platform-wide convention this repo established.

## Running locally

```bash
cargo run
```

Serves on `:8080`. See `CONTRIBUTING.md` for environment variables and the full local
development workflow.
