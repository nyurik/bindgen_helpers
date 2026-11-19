# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.3](https://github.com/nyurik/bindgen_helpers/compare/v0.6.2...v0.6.3) - 2026-11-19

### Other

- Update README.md
- limit job run time with timeout-minutes ([#53](https://github.com/nyurik/bindgen_helpers/pull/53))

## [0.6.2](https://github.com/nyurik/bindgen_helpers/compare/v0.6.1...v0.6.2) - 2026-11-19

### Added

- bindgen-helper as a CLI tool ([#42](https://github.com/nyurik/bindgen_helpers/pull/42))
- generate enums from C #define constants ([#38](https://github.com/nyurik/bindgen_helpers/pull/38))

### Fixed

- *(deps)* bound bindgen to <0.74 ([#50](https://github.com/nyurik/bindgen_helpers/pull/50))

### Other

- let release PRs pass the MSRV check ([#52](https://github.com/nyurik/bindgen_helpers/pull/52))
- release the CLI with the library, with binaries for six targets ([#51](https://github.com/nyurik/bindgen_helpers/pull/51))
- codecov project status off; justfile update uses cargo-upgrade ([#49](https://github.com/nyurik/bindgen_helpers/pull/49))
- [pre-commit.ci] pre-commit autoupdate ([#47](https://github.com/nyurik/bindgen_helpers/pull/47))
- fix justfile cargo binstall
- let rust fmt indent .rs files
- disallow mem leaking in code
- ignore CARGO_BUILD_WARNINGS in cargo-install
- use Rust 1.97 cargo warnings
- update .gitignore
- *(deps)* bump the all-actions-version-updates group across 1 directory with 2 updates ([#45](https://github.com/nyurik/bindgen_helpers/pull/45))
- update dependencies ([#43](https://github.com/nyurik/bindgen_helpers/pull/43))
- add descending value sort option ([#39](https://github.com/nyurik/bindgen_helpers/pull/39))
- add download counter to readme
- set release-plz CI ([#22](https://github.com/nyurik/bindgen_helpers/pull/22))
- validate if published in CI ([#21](https://github.com/nyurik/bindgen_helpers/pull/21))
- More rework of CI and justfiles ([#18](https://github.com/nyurik/bindgen_helpers/pull/18))
- coverage, updates ([#11](https://github.com/nyurik/bindgen_helpers/pull/11))
- Upgrade deps, use `Case<'static>`, lints ([#10](https://github.com/nyurik/bindgen_helpers/pull/10))
- Minor updates ([#7](https://github.com/nyurik/bindgen_helpers/pull/7))
- lints
- Reexport bindgen api to avoid conflicts ([#5](https://github.com/nyurik/bindgen_helpers/pull/5))
- regexset removals ([#3](https://github.com/nyurik/bindgen_helpers/pull/3))
- Update README.md
- Update README.md
- shields
- readme
- init

## [0.6.1](https://github.com/nyurik/bindgen_helpers/compare/v0.6.0...v0.6.1) - 2026-05-01

### Other

- add descending value sort option ([#39](https://github.com/nyurik/bindgen_helpers/pull/39))

## [0.6.0](https://github.com/nyurik/bindgen_helpers/compare/v0.5.1...v0.6.0) - 2026-05-01

### What's New
- generate enums from C #define constants ([#38](https://github.com/nyurik/bindgen_helpers/pull/38))

### Other
- improvements to CI and linting

## [0.5.1](https://github.com/nyurik/bindgen_helpers/compare/v0.5.0...v0.5.1) - 2025-10-01

### Other

- use automatic crates.io token
- add more linting
- *(just)* minor justfile cleanup
- *(ci)* improve cargo-install recipe
- *(ci)* disable telemetry in workflow
- improve get-create-field in justfile
- Bump actions/checkout from 4 to 5 in the all-actions-version-updates group ([#31](https://github.com/nyurik/bindgen_helpers/pull/31))
- *(ci)* format dependabot.yml
- [pre-commit.ci] pre-commit autoupdate ([#28](https://github.com/nyurik/bindgen_helpers/pull/28))
- *(ci)* do not fail ci on codecov
- *(ci)* update ci
- format Cargo.toml ([#29](https://github.com/nyurik/bindgen_helpers/pull/29))

## [0.4.6](https://github.com/nyurik/bindgen_helpers/compare/v0.4.5...v0.4.6) - 2025-06-11

### Other

- restrict bindgen to last non-breaking
- use release-plz token in dependabot ci

## [0.4.5](https://github.com/nyurik/bindgen_helpers/compare/v0.4.4...v0.4.5) - 2025-06-08

### Other

- consolidate release CI, dedup release and PR ([#25](https://github.com/nyurik/bindgen_helpers/pull/25))

## [0.4.4](https://github.com/nyurik/bindgen_helpers/compare/v0.4.3...v0.4.4) - 2025-06-06

- refactored CI, enabled release automation
