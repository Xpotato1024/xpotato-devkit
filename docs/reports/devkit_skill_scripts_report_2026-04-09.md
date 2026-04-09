# Devkit Skill Scripts Report (2026-04-09)

## Summary

- Added two bundled Python scripts for the highest-value deterministic skill workflows.

## Added Scripts

- `SKILLs/devkit-release-maintainer/scripts/check_release_version_alignment.py`
  - Checks that release tag injection and user-visible version wiring remain in place.
- `SKILLs/devkit-project-bootstrap/scripts/sync_repo_skills_to_codex.py`
  - Syncs the repo `SKILLs/` tree into the local Codex skill directory.

## Why These Two

- Release-version alignment is easy to regress and benefits from a repeatable static check.
- Local skill sync is operationally repetitive and easy to perform inconsistently by hand.

## Skill Updates

- Updated `devkit-release-maintainer` to call out the release alignment checker.
- Updated `devkit-project-bootstrap` to call out the skill sync script.

## Local Sync

- Re-synced the updated `SKILLs/` tree into `C:\Users\miyut\.codex\skills`.
- Verified that the installed copies include the new `scripts/` files.

## Remaining Risk

- These scripts intentionally cover only the most repetitive or fragile parts of the skill suite.
- Other skills still rely on references rather than scripts, which is acceptable until their workflows prove repetitive enough to justify code.
