# Tag-Triggered Release Implementation Report

Date: 2026-04-08

## Summary

Implemented a tag-triggered release flow for `xpotato-devkit`. Releases now publish only from `v*` tag pushes, with one GitHub Release containing all platform assets.

## Changed Files

- `.github/workflows/release.yml`
- `README.md`
- `docs/release/tag-triggered-release.md`

## What Changed

- Reworked the release workflow so it runs only on tag pushes matching `v*`.
- Split the workflow into a matrix build job and a single publish job.
- Packaged assets as `devkit-{tag}-{target}{ext}`.
- Switched the macOS release target to `x86_64-apple-darwin`.
- Added a maintainer-facing release guide under `docs/release/`.
- Added a short release note to `README.md` for user-facing clarity.

## Verification

- Reviewed the workflow diff and confirmed the release is no longer created from `main` or pull requests.
- Ran `git diff --check`.

## Notes

- No Rust source code changed.
- The release workflow now creates one GitHub Release per tag push and uploads all packaged artifacts to that release.
