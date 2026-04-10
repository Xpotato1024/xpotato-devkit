# Skill Script Repo Root Fix Report (2026-04-11)

## Summary

- Fixed repo-root auto-detection in bundled Python helper scripts so they work from this repository layout by default.
- Updated skill instructions to run helper scripts via `uv run python ...` instead of plain `python ...`.

## Changed Files

- `SKILLs/devkit-release-maintainer/scripts/check_release_version_alignment.py`
- `SKILLs/devkit-release-maintainer/SKILL.md`
- `SKILLs/devkit-release-maintainer/references/release-checklist.md`
- `SKILLs/devkit-project-bootstrap/scripts/sync_repo_skills_to_codex.py`
- `SKILLs/devkit-project-bootstrap/SKILL.md`

## Verification

- `uv run python SKILLs/devkit-release-maintainer/scripts/check_release_version_alignment.py`
- `uv run python SKILLs/devkit-project-bootstrap/scripts/sync_repo_skills_to_codex.py --codex-skills <temp-dir>`

## Notes

- The previous scripts assumed one extra parent directory above the repository root and failed when run from the checked-in `SKILLs/` tree.
- Both scripts now walk up from their own file path and stop at the first directory that matches the expected repository markers.
