# Release Version Metadata Report (2026-04-09)

## Summary

- Made release binaries version-aware via `DEVKIT_RELEASE_VERSION`.
- Aligned `devkit -V`, `devkit-installer --version`, and `install-manifest.json` version fields with the pushed release tag during release builds.

## Problem

- Release tags are currently managed independently from the crate `version` values in `Cargo.toml`.
- As a result, release binaries could report `0.1.0` even when shipped under a later tag such as `v0.1.3`.
- The installer manifest inherited the same mismatch because it stored `CARGO_PKG_VERSION`.

## Implementation

- Updated `rust/crates/devkit-cli/src/main.rs`
  - Clap version output now prefers `DEVKIT_RELEASE_VERSION`, with `CARGO_PKG_VERSION` as fallback.
- Updated `rust/crates/devkit-installer/src/main.rs`
  - Installer clap version output now prefers `DEVKIT_RELEASE_VERSION`.
  - `install-manifest.json` now records `version` and `installer_version` from `DEVKIT_RELEASE_VERSION` when present.
- Updated `.github/workflows/release.yml`
  - Release builds now export `DEVKIT_RELEASE_VERSION=${{ github.ref_name }}` for all target jobs.

## Verification

- Local default build still reports crate-version fallback behavior.
- Local builds with `DEVKIT_RELEASE_VERSION=v9.9.9` report the injected release version for:
  - `devkit -V`
  - `devkit-installer --version`

## Remaining Risk

- Non-release local builds still fall back to crate version, which is intentional.
- If another external packaging path is added later, it must also provide `DEVKIT_RELEASE_VERSION` if release-tag-aligned version output is required.
