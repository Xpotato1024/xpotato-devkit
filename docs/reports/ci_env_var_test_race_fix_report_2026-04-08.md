# CI Env Var Test Race Fix Report

Date: 2026-04-08

## Summary

Fixed the remaining CI failure on `Test (macos-latest)` caused by test-level environment variable races around `DEVKIT_CONFIG`.

## Root Cause

- `devkit-core` includes a test that sets `DEVKIT_CONFIG` to verify explicit config override behavior.
- Other tests in the same test binary also call `load_config()` and assume no explicit override is active.
- Rust test binaries run tests in parallel by default, so these tests could overlap and produce nondeterministic results.
- The failure surfaced on macOS CI in `loads_metrics_config_from_parent_root`, where `config.metrics.enabled` unexpectedly stayed false.

## Changes

- Added a shared test-only mutex in `devkit-core` to serialize tests that read or write `DEVKIT_CONFIG`.
- Added the same guard pattern in `devkit-metrics` for the env-var-based config path test.

## Files Changed

- `rust/crates/devkit-core/src/lib.rs`
- `rust/crates/devkit-metrics/src/lib.rs`

## Verification

- `cargo test -p devkit-core -p devkit-metrics`
- `cargo test -p devkit-block -p devkit-core -p devkit-metrics -p devkit-patch -p devkit-cli`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Remaining Risk

- Any future test that mutates process-wide environment variables must use the same serialization pattern or it can reintroduce nondeterministic CI failures.
