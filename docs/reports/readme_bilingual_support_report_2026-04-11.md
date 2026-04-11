# README Bilingual Support Report (2026-04-11)

## Summary

- Added language-switch navigation for the repository README.
- Preserved the existing README content as `README.ja.md`.
- Replaced the root `README.md` with an English README that links to the Japanese version.

## Updated Files

- `README.md`
- `README.ja.md`
- `docs/reports/readme_bilingual_support_report_2026-04-11.md`

## What Changed

- Added `English | 日本語` language links at the top of the root README flow.
- Kept the existing Japanese README content in a dedicated `README.ja.md` file.
- Rewrote the root `README.md` as an English entry point for GitHub and package-facing readers.
- Kept the main user-facing sections aligned across the two language variants:
  - overview
  - installation
  - common commands
  - release notes
  - Windows installation

## Verification

- Ran `devkit encoding check README.md README.ja.md docs/reports/readme_bilingual_support_report_2026-04-11.md --brief`.
- Reviewed the staged diff to confirm the README split is limited to language-switch support and documentation updates.

## Notes

- The Japanese README remains the closest representation of the previous top-level README content.
- The English README is intentionally concise and optimized for the default GitHub landing page.
