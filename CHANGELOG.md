
## 0.1.1 - 2026-07-27

### Fixed
- Release workflow permissions, restore debug.yml, fresh k8s manifests
- Add missing card back images
- Add card background



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
