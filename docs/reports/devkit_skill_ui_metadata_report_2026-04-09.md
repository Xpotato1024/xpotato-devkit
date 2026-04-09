# Devkit Skill UI Metadata Report (2026-04-09)

## Summary

- Added `agents/openai.yaml` metadata for all repo-bundled `devkit` skills.
- Re-synced the updated skills into the local Codex skill store so the installed copies include the same metadata.

## Updated Skills

- `devkit-doc-edit`
- `devkit-encoding-hygiene`
- `devkit-git-drafts`
- `devkit-inspect-edit-verify`
- `devkit-metrics-review`
- `devkit-project-bootstrap`
- `devkit-release-maintainer`
- `devkit-tree-explore`

## Metadata Added

Each skill now has:

- `interface.display_name`
- `interface.short_description`
- `interface.default_prompt`

## Purpose

- Improve discoverability in skill lists and invocation chips.
- Make each skill easier to invoke explicitly with a reasonable starter prompt.
- Keep the repo-bundled skills closer to the structure used by installed Codex skills.

## Verification

- Synced `SKILLs/` into `C:\Users\miyut\.codex\skills`
- Verified repo and installed copies match after sync

## Remaining Risk

- The skills still use minimal metadata only. Icons, colors, and richer packaged references can be added later if they become worth the maintenance cost.
