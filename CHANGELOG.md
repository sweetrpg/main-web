
## Unreleased

### Added
- Read-only shared session support: shows a "log in"/"log out" link in the nav, reading the
  session `auth-web` establishes (Redis-backed, fail-open on a Redis outage). No Auth0 code of
  its own - `auth-web` is the suite's sole login owner.

### Fixed
- `build_hash` no longer panics on the local-dev placeholder value (shorter than the template's
  8-character slice); the placeholder itself is now `deadbeef` instead of `unset`.

### Changed
- Renamed the `hub`/`Hub` CSS classes, template IDs, and `HubTemplate` struct to
  `landing`/`Landing` - this app is a landing page, not a hub.

## 0.1.2 - 2026-07-28

### Fixed
- Remove HPA and PDB from dev overlay



## 0.1.1 - 2026-07-27

### Fixed
- Release workflow permissions, restore debug.yml, fresh k8s manifests
- Add missing card back images
- Add card background



## 0.1.0 - 2026-07-27

### Fixed
- Release workflow permissions, restore debug.yml, fresh k8s manifests



## 0.1.0 - 2026-07-27

### Added
- Rewrite main-web as an Axum/Askama Rust service


### Fixed
- Secret version
- Point prepare-release at the new Main Web project


# Changelog

## Unreleased

### Added

- Rewrite as an Axum/Askama Rust web service, replacing the dormant, never-deployed
  Python/Flask skeleton. Serves the SweetRPG Hub landing page at `/`.
