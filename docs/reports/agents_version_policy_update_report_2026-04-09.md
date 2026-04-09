# AGENTS Version Policy Update Report (2026-04-09)

## Summary

- Updated `AGENTS.md` to require strict alignment between tagged releases and user-visible version metadata.

## Changes

- Added CLI design guidance that user-visible version reporting must follow release metadata rather than assuming crate metadata is sufficient.
- Added an explicit rule that tagged releases must align:
  - `devkit -V`
  - installer `--version`
  - release artifacts
  - installer manifests
- Added verification guidance requiring both local fallback checks and tagged-release-path checks when release metadata changes.

## Purpose

- Prevent regressions where release artifacts are published under one tag while the CLI or installer still reports the crate-local version.
- Make the release-version contract explicit for future agents and maintainers.
