# Windows Release Hotfix Report (2026-04-09)

## Summary

- Fixed the Windows release workflow so the dedicated cleanup helper is built first without embedded-payload environment variables.
- Kept payload embedding only on the `devkit-installer` build invocation.

## Root Cause

- The release workflow exported `DEVKIT_INSTALLER_PAYLOAD` and `DEVKIT_CLEANUP_HELPER` for the whole "Build Windows installer" step.
- That environment was also visible while building `devkit-cleanup-helper`.
- `rust/crates/devkit-installer/build.rs` therefore tried to embed `devkit-cleanup-helper.exe` before that file existed and aborted with:
  - `DEVKIT_CLEANUP_HELPER does not point to a file`

## Implementation

- Updated `.github/workflows/release.yml`.
- The first `cargo build` now builds `devkit-cleanup-helper` with no embedding environment variables.
- The workflow sets `DEVKIT_INSTALLER_PAYLOAD` and `DEVKIT_CLEANUP_HELPER` only immediately before building `devkit-installer`.

## Verification

- Reproduced the intended local sequence:
  - build `devkit-cli`
  - build `devkit-cleanup-helper` without embedding env vars
  - build `devkit-installer` with both embedding env vars set

## Remaining Risk

- The fix is scoped to the release workflow. If a future workflow or local script again exports embedding variables too early, the same class of failure can recur.
