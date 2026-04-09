---
name: devkit-release-maintainer
description: Use when an AI agent needs to prepare, verify, or debug devkit releases, including tags, release metadata, installer packaging, and user-visible version alignment.
---

# devkit release maintainer

Use this skill for release-oriented work on `devkit` itself.

## When to use

- A release tag is about to be cut or was just cut.
- Release metadata, installer manifests, or version display may be inconsistent.
- A GitHub Actions release build or packaging flow needs verification.
- Windows installer packaging behavior needs release validation.

## Workflow

1. Confirm release-facing metadata.
   - Check `README.md`, `docs/release/`, and `AGENTS.md`.
   - Verify user-visible version paths such as `devkit -V`, installer `--version`, and manifest version fields.
2. Verify local build behavior.
   - `cargo test -p devkit-cli -p devkit-installer`
   - `cargo run -p devkit-cli -- -V`
   - For release-injected behavior, build or run with `DEVKIT_RELEASE_VERSION=<tag>`
3. Verify installer and packaging flow when relevant.
   - Build `devkit-cli`
   - Build `devkit-cleanup-helper`
   - Build `devkit-installer` with embedded payload env vars
4. Only then proceed to tag / release publication.

## Rules

- Treat release tags as the source of truth for user-facing release version output.
- Verify both fallback local behavior and tagged-release behavior when version metadata is touched.
- Do not assume crate `version` fields alone define the shipped release version.

## Reference

- See `docs/release/`
- See `docs/reports/release_version_metadata_report_2026-04-09.md`
- When actively preparing or validating a release, read [references/release-checklist.md](references/release-checklist.md).
