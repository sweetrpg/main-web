# AGENTS.md

This file provides guidance to Claude Code, Codex, GitHub Copilot, and other coding agents
working in this repository.

## About This Project

`main-web` is the landing page for the SweetRPG suite, serving `dev.sweetrpg.com/` (and
`sweetrpg.com/` in production): suite branding, a light/dark/system theme toggle, and a card
grid linking out to each app in the suite (`Catalogue` live via `catalog-web`'s `/catalog` path,
`Shelf`/`Systems`/`Profile`/`Initiative!` marked "Coming soon" until those frontends exist).

It's the org's first Rust **web service** (as opposed to the existing Rust *library* crates -
`common.rs`, `model-core.rs`, `catalog-objects.rs`, etc.) and the reference implementation for
Rust web-service conventions on this platform - see `sweetrpg/platform`'s
`docs/rust-service-conventions.md` for the platform-wide write-up this repo established. The
choices below are specific to this repo; treat them as the baseline for the next Rust web
service, not an existing platform-wide standard being followed.

This replaced a dormant Python/Flask skeleton that was never deployed - no live traffic, no
data migration, a clean rewrite rather than an incremental port.

### Framework and templating

- **Axum**, chosen over Actix-web/Rocket for being Tokio-native and having first-class `tower`
  middleware compatibility (tracing, metrics).
- **Askama** for templates (`templates/hub.html`) - compile-time-checked: a template referencing
  a field the context struct doesn't have fails the build, not a request.
- The app card list (`src/routes/hub.rs`) is a compile-time `Vec<AppCard>`, not a config file -
  adding a new app already requires a code change (its `href` moving off `#`) and redeploy
  either way, so a config file would add indirection without removing a deploy step.

### Shared static assets

Suite-wide branding (logo variants, favicon, Broadsheet's design tokens) is served by
`assets-web`, not duplicated here - see `sweetrpg/platform`'s `docs/frontend-conventions.md`.
Referenced through `SHARED_ASSETS_URL` (`src/config.rs`), never a hardcoded host - defaults to
a local `assets-web` instance's own address for local development.

This app's own `static/` directory holds only what's genuinely local to this page: the
extracted theme-toggle JS (`theme.js`), the adapted Broadsheet CSS (`broadsheet.css`), and the
Hub page's own layout CSS (`hub.css`).

### Build info / version footer

`src/build_info.rs` reads `BUILD_INFO_PATH` (default `/app/config/build-info.json`), a file the
`Dockerfile` bakes in at image build time from `BUILD_NUMBER`/`BUILD_JOB`/`BUILD_SHA`/
`BUILD_DATE`/`BUILD_VERSION` build args - matches `catalog-web`'s `BuildInfo.swift` pattern.
Falls back to placeholder values (`version: "dev"`) when the file doesn't exist, e.g. local
`cargo run` outside a container.

## Observability

- **Logging**: `tracing` + `tracing-subscriber`'s JSON formatter, one object per line to stdout.
  `LOG_LEVEL` env var (`tracing`/`EnvFilter` syntax, e.g. `info`, `debug`), default `info`.
- **Tracing**: OTLP/HTTP via `opentelemetry-otlp` (`src/telemetry.rs`), to
  `OTEL_EXPORTER_OTLP_ENDPOINT`. The tracer provider is built once at startup and its shutdown
  is deferred to the very end of `main` (after `axum::serve` returns) rather than dropped early -
  a provider dropped early silently stops exporting spans, the same failure mode `assets-web`'s
  `application/tracing.py` documents hitting catalog-api via a misplaced `defer`. No-op (not a
  startup failure) when the env var is unset, so local dev doesn't require a collector.
- **Metrics**: `axum-prometheus` at `/metrics`, Prometheus text exposition format - scraped via
  the platform's standard `PodMonitor` pattern.
- **Health checks**: `/healthz` and `/readyz`, both a bare 200 - this service has no
  database/cache dependency whose health would need reflecting separately.

## Committing Code

[Conventional Commits](https://www.conventionalcommits.org/): `<type>(<scope>): <description>`.

## Branches and Workflow

Git-flow (see `docs/git-flow.md` in `sweetrpg/platform`): `develop` is the integration branch,
`master` reflects the latest release. Feature/fix branches off `develop`, PR back into `develop`.

Releasing: dispatch the "Prepare Release" workflow - it computes the next version via
`git-cliff`, bumps `Cargo.toml`/`Cargo.lock`, updates `CHANGELOG.md`, and opens a
`release/<version>` PR into `master` (using `sweetrpg/github-actions`'s reusable
`rust-prepare-release.yaml`/`rust-release.yaml`/`rust-tag-release.yaml` workflows - this repo is
their first real consumer). Merging that PR tags the release; `docker-build.yml` then builds and
pushes `ghcr.io/sweetrpg/main-web` and bumps the deployed image tag in `sweetrpg/kubernetes` via
`argocd-update-deployment.yaml`. No crates.io publish - `rust-release.yaml`'s
`publish-to-crates-io` is set `false` here, since this ships only as a container image, not a
library.

## Running Checks Locally

```bash
cargo fmt --check
cargo clippy --all-targets
cargo test
cargo run
```

`cargo run` serves on `:8080`. Set `SHARED_ASSETS_URL` to a reachable `assets-web` instance
(e.g. `https://dev.sweetrpg.com/assets`) to see the real logo/branding rendered locally instead
of broken image references.
