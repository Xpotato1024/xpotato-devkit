# Windows Installation User Flow Report

Date: 2026-04-08

## Summary

Implemented a native Windows installer flow for `devkit`. The release archive now ships `devkit-installer.exe` alongside `devkit.exe`, and the installer writes the local install manifest and optional PATH entry without relying on PowerShell scripts.

## Changed Files

- `rust/crates/devkit-installer/Cargo.toml`
- `rust/crates/devkit-installer/src/main.rs`
- `.github/workflows/release.yml`
- `docs/install/windows-installation.md`
- `docs/release/tag-triggered-release.md`
- `docs/release/release-process.md`
- `README.md`

## What Changed

- Added a new Rust crate for the Windows installer executable.
- The installer:
  - copies the bundled `devkit.exe` into `%LOCALAPPDATA%\Xpotato\devkit`
  - copies itself to `uninstall.exe`
  - writes `install-manifest.json`
  - optionally adds the install directory to the user PATH
  - removes only manifest-recorded files during uninstall
- Updated the Windows release packaging so the archive includes both the CLI payload and the native installer.
- Reworked the user-facing install documentation to match the native installer flow.

## Verification

- Ran `git diff --check`.
- Ran `cargo test -p devkit-installer`.
- Ran `cargo build -p devkit-installer --release`.
- Added unit tests for manifest round-tripping and path normalization helpers in the installer crate.

## Notes

- No PS1-based installer remains in the repository.
- The installer is intentionally scoped to user-local installation only.
