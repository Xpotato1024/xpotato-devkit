# CI Test Hardening Report

Date: 2026-04-08

## Summary

Hardened the Rust test suite against repeated CI-only flakes instead of applying another single-failure patch.

## Root Causes Addressed

- Process-wide environment variable races around `DEVKIT_CONFIG`.
- Temporary-directory name collisions caused by timestamp-only test paths.
- OS-specific path alias differences such as macOS `/var/...` vs `/private/var/...`.

## Changes

- Serialized tests that mutate `DEVKIT_CONFIG`.
- Replaced timestamp-only temp directory names with `pid + timestamp + atomic counter` patterns in Rust test helpers across multiple crates.
- Replaced string-based metrics path assertions with same-location checks based on canonicalized parent directories plus filename comparison.

## Files Changed

- `rust/crates/devkit-core/src/lib.rs`
- `rust/crates/devkit-encoding/src/lib.rs`
- `rust/crates/devkit-metrics/src/lib.rs`
- `rust/crates/devkit-bootstrap/src/lib.rs`
- `rust/crates/devkit-block/src/lib.rs`
- `rust/crates/devkit-installer/src/manifest.rs`
- `rust/crates/devkit-installer/src/main.rs`

## Verification

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- Repeated 5x:
  - `cargo test -p devkit-core -p devkit-encoding -p devkit-metrics -p devkit-bootstrap -p devkit-block -p devkit-installer`

## Remaining Risk

- Any new test that uses process-wide environment variables or string-comparison of platform-dependent paths can reintroduce CI-only flakes if it does not follow the same patterns.
