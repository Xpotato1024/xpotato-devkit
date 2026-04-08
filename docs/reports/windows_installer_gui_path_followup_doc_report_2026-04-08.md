# Windows Installer GUI/PATH Follow-up Doc Report

Date: 2026-04-08

## Summary

Added a Japanese follow-up planning document for Windows installer behavior after confirming that GUI execution can complete without adding the install directory to PATH and may coexist with an older `devkit.exe` on the user PATH.

## Added Files

- `docs/proposals/windows_installer_gui_path_and_signing_followup_2026-04-08.md`

## Covered Topics

- Current installer behavior and its mismatch with GUI user expectations
- PATH handling as the default install behavior
- Coexistence and conflict with `C:\Users\miyut\.local\bin\devkit.exe`
- Single-file installer direction
- Code-signing and SmartScreen considerations
- Remaining tasks and acceptance criteria in checklist form

## Verification

- `cargo run -p devkit-cli -- encoding check <doc files> --brief`
- `git diff --check`

## Notes

- This branch adds documentation only.
- No installer behavior changes are implemented yet in this branch.
