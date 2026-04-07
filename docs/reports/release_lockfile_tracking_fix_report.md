# Release Lockfile Tracking Fix Report

Date: 2026-04-08

## Summary

Fixed the release build failure caused by `rust/Cargo.lock` being ignored and therefore missing from the GitHub Actions checkout.

## Changed Files

- `rust/.gitignore`
- `rust/Cargo.lock`
- `docs/reports/release_lockfile_tracking_fix_report.md`

## What Changed

- Removed the `Cargo.lock` ignore rule from `rust/.gitignore`.
- Added the Rust workspace lockfile to the repository so `cargo build --locked` can run in CI and release workflows.

## Verification

- Confirmed `rust/Cargo.lock` is now tracked in Git.
- Confirmed the release workflow failure was caused by a missing lockfile in the checked-out repository.

## Notes

- This fix is limited to release reproducibility and does not change CLI behavior.
