# Windows Installer Follow-up Report

Date: 2026-04-08

## Summary

Refined the existing `devkit-installer` implementation so install and uninstall behavior follows the manifest more closely and handles Windows-specific cleanup more predictably.

## Changed Files

- `rust/crates/devkit-installer/src/main.rs`
- `docs/reports/windows_installer_followup_report.md`

## What Changed

- Added an internal `InstallPaths` helper to centralize install-path derivation.
- Changed uninstall manifest resolution so a generated `uninstall.exe` reads `install-manifest.json` from its own directory by default instead of assuming the default install root.
- Made uninstall use manifest-recorded install and PATH information as the primary source of truth.
- Added empty-directory cleanup after uninstall, while keeping self-delete on `uninstall.exe`.
- Changed manifest writes to go through a temporary file before replacing the previous manifest file.
- Split manifest handling and PATH management into dedicated modules instead of keeping the entire implementation in `main.rs`.
- Added Windows environment change broadcasting after PATH updates so newly launched shells can observe the change more reliably.
- Added unit tests for manifest path resolution, install directory resolution, and PATH removal metadata handling.

## Verification

- Ran `cargo test -p devkit-installer`.
- Ran `cargo build -p devkit-installer --release`.

## Remaining Risks

- PATH updates still rely on registry writes only and do not broadcast a Windows environment change notification.
- The installer logic is more structured, but it still lives in `main.rs`; a further module split may be worthwhile if the crate grows.
