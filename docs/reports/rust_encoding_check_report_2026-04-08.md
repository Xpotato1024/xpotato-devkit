# Rust Encoding Check Implementation Report

Date: 2026-04-08

## Summary

Implemented Rust support for `devkit encoding check` and wired it into the public CLI.

## Scope

- Added a dedicated `devkit-encoding` crate for reusable encoding-check logic.
- Added `devkit encoding check <files...>` to the Rust CLI.
- Preserved config-driven ignore handling via `devkit.toml` `encoding.ignore`.
- Added brief-mode summary output and deterministic standard-mode result lines.
- Updated public docs to show `encoding check` as supported by the Rust binary.

## Acceptance Criteria Met

- `devkit encoding check <files...>` runs in Rust.
- Detects invalid UTF-8, BOM, replacement characters, control characters, and mixed newlines.
- Uses `encoding.ignore` from config, with legacy-compatible default ignore patterns when unset.
- `--brief` emits a single `OK:` / `FAIL:` line.
- Exit code is `1` when issues are detected or no valid files are processed.

## Files Changed

- `rust/crates/devkit-encoding/Cargo.toml`
  - New crate definition for encoding checks.
- `rust/crates/devkit-encoding/src/lib.rs`
  - Input expansion, ignore filtering, encoding inspection, and unit tests.
- `rust/crates/devkit-cli/Cargo.toml`
  - Added dependency on `devkit-encoding`.
- `rust/crates/devkit-cli/src/main.rs`
  - Added `encoding check` subcommand wiring and output formatting.
- `README.md`
  - Marked `encoding check` as currently supported and added a usage example.
- `docs/rust-parity-matrix.md`
  - Recorded Rust parity for `encoding check`.

## Verification

- `cargo fmt --all`
- `cargo test -p devkit-encoding -p devkit-cli`

## Remaining Gaps

- `devkit encoding normalize` is still not implemented in Rust.
- Rust still does not provide `bootstrap` or `metrics`.
- Legacy block and patch interface parity work remains unchanged.
