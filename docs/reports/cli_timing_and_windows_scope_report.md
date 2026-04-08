# CLI Timing And Windows Scope Report

Date: 2026-04-08

## Summary

Clarified that the native installer is Windows-only while Linux and macOS releases remain archive-only, and fixed the Rust CLI so `--time` and `--time-json` emit total execution timing.

## Changed Files

- `rust/crates/devkit-cli/src/main.rs`
- `README.md`
- `docs/release/tag-triggered-release.md`
- `docs/reports/cli_timing_and_windows_scope_report.md`

## What Changed

- Added total timing output for the Rust CLI global flags:
  - `--time` prints human-readable timing to stderr
  - `--time-json` prints JSON timing to stderr
- Added unit tests for timing output formatting.
- Updated user-facing documentation to state that the native installer is included only in Windows release archives.
- Clarified that Linux and macOS release archives ship the `devkit` binary only.

## Verification

- Ran `cargo test -p devkit-cli`.
- Ran `cargo run -p devkit-cli -- --time tree --path .`.
- Ran `cargo run -p devkit-cli -- --time-json tree --path .`.

## Remaining Risks

- The Rust CLI timing output currently reports total elapsed time only; category breakdowns such as `git_ms` or `io_ms` are not implemented yet.
