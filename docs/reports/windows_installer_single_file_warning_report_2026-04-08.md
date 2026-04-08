# Windows Installer Single-File And Warning Report

Date: 2026-04-08

## Summary

Extended the Windows installer follow-up work to cover single-file distribution and clearer PATH conflict messaging.

- `devkit-installer.exe` can now carry an embedded `devkit.exe` payload
- `devkit-installer.exe` can now carry an embedded cleanup helper payload
- the Windows release workflow now builds the installer with the embedded payload
- the Windows release archive now contains `devkit-installer.exe` only
- install output now reports payload source and explains whether the current shell or a new shell is expected to resolve the installed binary
- the follow-up proposal checklist was updated to reflect the implemented items

## Changed Files

- `rust/crates/devkit-installer/build.rs`
- `rust/crates/devkit-installer/src/bin/devkit-cleanup-helper.rs`
- `rust/crates/devkit-installer/src/main.rs`
- `.github/workflows/release.yml`
- `README.md`
- `docs/install/windows-installation.md`
- `docs/release/tag-triggered-release.md`
- `docs/release/release-process.md`
- `docs/proposals/windows_installer_gui_path_and_signing_followup_2026-04-08.md`
- `docs/reports/windows_installer_single_file_warning_report_2026-04-08.md`

## Implementation Details

- Added a build script that embeds a `devkit.exe` payload into the installer when `DEVKIT_INSTALLER_PAYLOAD` is provided at build time.
- Added a dedicated `devkit-cleanup-helper` binary and embed path for it via `DEVKIT_CLEANUP_HELPER`.
- Kept a sidecar fallback so local development builds still work when the payload env var is not set.
- Changed install flow to write the embedded payload directly to `%LOCALAPPDATA%\Xpotato\devkit\devkit.exe`.
- Replaced the temporary copied installer cleanup helper with a dedicated tiny helper executable.
- Added a reboot-time delete fallback via `MoveFileExW(..., MOVEFILE_DELAY_UNTIL_REBOOT)` when immediate cleanup still fails.
- Added a cleanup helper log at `%TEMP%\devkit-cleanup-helper.log` for fallback cases.
- Added resolution messaging that distinguishes current-shell resolution from expected new-shell resolution.
- Improved warnings so they show the conflicting `devkit.exe` path instead of a generic PATH conflict notice.
- Added a Windows E2E round-trip test that copies the installer and sidecar payload into a temporary package directory, installs with `--unpack-only`, validates the manifest, and then runs `uninstall.exe`.
- Updated release packaging to stop shipping a separate `devkit.exe` in the Windows archive.
- Updated the proposal checklist to mark completed PATH, conflict-detection, single-file-installer, and doc tasks.

## Verification

- `cargo fmt --all`
- `cargo test -p devkit-installer`
- `cargo test -p devkit-installer --test e2e`
- `cargo build -p devkit-installer --release`
- `cargo +stable-x86_64-pc-windows-msvc build -p devkit-cli --release --target x86_64-pc-windows-msvc`
- `cargo +stable-x86_64-pc-windows-msvc build -p devkit-installer --release --target x86_64-pc-windows-msvc --bin devkit-cleanup-helper`
- `$env:DEVKIT_INSTALLER_PAYLOAD=(Resolve-Path '<repo>\\rust\\target\\x86_64-pc-windows-msvc\\release\\devkit.exe').Path; $env:DEVKIT_CLEANUP_HELPER=(Resolve-Path '<repo>\\rust\\target\\x86_64-pc-windows-msvc\\release\\devkit-cleanup-helper.exe').Path; cargo +stable-x86_64-pc-windows-msvc build -p devkit-installer --release --target x86_64-pc-windows-msvc --bin devkit-installer`
- `cargo run -p devkit-cli -- encoding check ..\README.md ..\docs\install\windows-installation.md ..\docs\release\tag-triggered-release.md ..\docs\release\release-process.md ..\docs\proposals\windows_installer_gui_path_and_signing_followup_2026-04-08.md ..\docs\reports\windows_installer_single_file_warning_report_2026-04-08.md --brief`
- `git diff --check`

## Remaining Risks

- The generated uninstaller cleanup is now retry-based, has a reboot fallback, and is covered by E2E, but it still relies on spawning a temporary helper executable.
- Signing and SmartScreen mitigation are still intentionally deferred.
