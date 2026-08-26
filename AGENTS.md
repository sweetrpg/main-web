# AGENTS.md

This file provides guidance to Claude Code, Codex, GitHub Copilot, and other coding agents
working in this repository.

## About This Project

`main-web` is the landing page for the SweetRPG suite, serving `dev.sweetrpg.com/` (and
`sweetrpg.com/` in production): suite branding, a light/dark/system theme toggle, and a card
grid linking out to each app in the suite (`Catalogue` live via `catalog-web`'s `/catalog` path,
`Game Room`/`Systems`/`Profile`/`Initiative!` marked "Coming soon" until those frontends exist).

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
- **Askama** for templates (`templates/main.html`) - compile-time-checked: a template referencing
  a field the context struct doesn't have fails the build, not a request.
- The app card list (`src/routes/main.rs`) is a compile-time `Vec<AppCard>`, not a config file -
  adding a new app already requires a code change (its `href` moving off `#`) and redeploy
  either way, so a config file would add indirection without removing a deploy step.

### Localization

User-facing strings come from `locales/<code>.yml` via `rust-i18n`, never hardcoded in
templates. English is the default/fallback locale; add a new locale by creating
`locales/<code>.yml` and adding its code to `SUPPORTED_LOCALES` in `src/i18n.rs`. Locale
resolution per request: `locale` cookie override, then `Accept-Language` (first tag, base
subtag matched), then English - see the `web-frontend-localization` spec in
`sweetrpg/platform`'s `openspec/changes/full-localization-web-apps`.

Askama can't invoke the `t!` macro directly, so templates receive a `Tr` translator
(`src/i18n.rs`) and call its methods (`{{ tr.menu_log_in() }}`). Dynamic keys go through
`tr.get("cards.<id>.name")`. Locale files are embedded at compile time; `build.rs` re-runs
the `i18n!` expansion when anything under `locales/` changes. CI runs
`scripts/check-template-strings.sh` (`locale-lint` job), which fails on literal text
between HTML tags that isn't a whitelisted brand string or sourced from `{{ tr.* }}`.

### Shared static assets

Suite-wide branding (logo variants, favicon, Broadsheet's design tokens) is served by
`shared-web`, not duplicated here - see `sweetrpg/platform`'s `docs/frontend-conventions.md`.
Referenced through `SHARED_URL` (`src/config.rs`), never a hardcoded host - defaults to
a local `shared-web` instance's own address for local development.

This app's `static/` directory holds only `img/` (the app-card background images) - the
theme-toggle JS, Broadsheet CSS, and this page's own landing layout all moved into `shared-web`'s
`static/css/main.css`/`static/js/theme.js` and are pulled in via `SHARED_URL` (see
`sweetrpg/platform`'s `openspec/changes/consolidate-shared-design-system`).

### Build info / version footer

`src/build_info.rs` reads `BUILD_INFO_PATH` (default `/app/config/build-info.json`), a file the
`Dockerfile` bakes in at image build time from `BUILD_NUMBER`/`BUILD_JOB`/`BUILD_SHA`/
`BUILD_DATE`/`BUILD_VERSION` build args - matches `catalog-web`'s `BuildInfo.swift` pattern.
Falls back to placeholder values (`version: "dev"`) when the file doesn't exist, e.g. local
`cargo run` outside a container.

### Banner messages (`admin_client.rs`)

`AdminClient` fetches platform-wide banner messages from `admin-api` (`sweetrpg/admin-api`) for
the `platform` and `service:main` scopes, per the `add-banner-messages` OpenSpec change
(`sweetrpg/platform`). Disabled by default - `base_url` is `None` unless `ADMIN_API_URL` is set,
so a deploy with the var unset makes zero network calls. When enabled: a 90s in-memory cache
(one `Mutex<Option<CacheEntry>>`, not per-scope-key - fine at this service's read volume) and a
2s request timeout. Fails open on every error path (timeout, connection error, non-200, bad
JSON) by returning an empty `Vec<Banner>` - a banner outage must never break the landing page.

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

`cargo run` serves on `:8080`. Set `SHARED_URL` to a reachable `shared-web` instance
(e.g. `https://dev.sweetrpg.com/shared`) to see the real logo/branding rendered locally instead
of broken image references.
