
## 0.3.0 - 2026-08-04

### Added
- Replace plain-text user identity with the shared avatar menu
- Add guarded Sentry error reporting
- Add maintenance badge to app cards
- Add TLS for local ingress


### Fixed
- Cap banner width to 80% of viewport on landing page
- Strip leading v from tagged build version to avoid double-v footer
- Render a specific message per login-error reason
- Authenticate to the shared session Redis
- Rename avatar menu's Admin link to Administration
- Set ADMIN_API_URL so banners/maintenance-mode badges render



## 0.2.2 - 2026-08-02


## 0.2.2 - 2026-08-02

### Fixed
- Keep theme menu closed until toggled (#165)
- Surface auth-web's login_error redirect as a banner



## 0.2.1 - 2026-08-01

### Fixed
- Scope card background image to its own card
- Point shared session Redis at the correct host/DB (#163)



## 0.2.0 - 2026-08-01

### Added
- Add AdminClient for banner message integration (#155)
- Truncate build hash to 8 characters in footer
- Fix card backgrounds and improve UI (#159)
- Read-only shared session, rename hub to landing, fix build hash panic (#160)


### Documentation
- Add coverage badge, ci: build arm64 image alongside amd64


### Fixed
- Correct memory quantity suffix from milli to mebibytes (#156)
- Ingress config
- Card backgrounds not loading
- Remove hover underline from card links



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
