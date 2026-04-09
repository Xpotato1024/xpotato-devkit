# Windows Installer GUI/PATH Follow-up Implementation Report

Date: 2026-04-08

## Summary

Implemented the first follow-up slice from the Windows installer GUI/PATH proposal:

- the installer now adds the install directory to the user PATH by default
- `--unpack-only` is the explicit opt-out for PATH updates
- install output now reports PATH status and reminds users to open a new shell
- the installer warns when another `devkit.exe` is already resolved from the current PATH

This patch does not implement single-file installer bundling or code-signing workflow changes.

## Changed Files

- `rust/crates/devkit-installer/src/main.rs`
- `docs/install/windows-installation.md`
- `README.md`
- `docs/reports/windows_installer_gui_path_followup_implementation_report_2026-04-08.md`

## Implementation Details

- Changed installer PATH behavior from opt-in to default-on.
- Kept `--add-to-path` accepted for compatibility, but made `--unpack-only` the only explicit PATH opt-out.
- Added `PathStatus` handling so install output distinguishes `Added`, `Already present`, `Skipped (--unpack-only)`, and `Failed`.
- Added PATH conflict detection against the current shell PATH and emit warnings when another `devkit.exe` is found first.
- Added unit tests for default PATH behavior, `--unpack-only` behavior, and PATH command discovery.
- Updated Windows installation docs and README to match the new installer behavior.

## Verification

- `cargo fmt --all`
- `cargo test -p devkit-installer`
- `cargo build -p devkit-installer --release`
- `cargo run -p devkit-cli -- encoding check ..\README.md ..\docs\install\windows-installation.md ..\docs\reports\windows_installer_gui_path_followup_implementation_report_2026-04-08.md --brief`
- `git diff --check`

## Remaining Risks

- PATH conflict detection checks the current process PATH and warns conservatively; it does not fully predict every future shell resolution order.
- The Windows release is still an extracted bundle containing both `devkit-installer.exe` and `devkit.exe`.
- Code-signing and SmartScreen mitigation are still documentation/planning items only.
