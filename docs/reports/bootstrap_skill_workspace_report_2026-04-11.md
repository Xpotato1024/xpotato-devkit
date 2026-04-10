# Bootstrap Skill Workspace Report (2026-04-11)

## Summary

- Added `devkit bootstrap sync-skills` to copy repo-bundled `SKILLs/` into a target workspace.
- Added `devkit bootstrap init-agents` to generate a starter `AGENTS.md` for workspace-local skill usage.
- Kept this behavior out of the Windows installer so the installer remains a generic CLI distribution path.

## Changed Files

- `rust/crates/devkit-bootstrap/src/lib.rs`
- `rust/crates/devkit-cli/src/main.rs`
- `rust/crates/devkit-cli/tests/cli_smoke.rs`
- `README.md`
- `SKILLs/devkit-project-bootstrap/SKILL.md`
- `SKILLs/devkit-project-bootstrap/references/setup-checklist.md`
- `docs/rust-parity-matrix.md`

## Verification

- `cargo test --manifest-path rust/Cargo.toml -p devkit-bootstrap -p devkit-cli`
- CLI smoke covers:
  - `bootstrap sync-skills`
  - `bootstrap init-agents`

## Notes

- `sync-skills` defaults to copying into `<target>/SKILLs`.
- `init-agents` defaults to writing `AGENTS.md` in the current directory and lists detected bundled skills when the configured skills directory already exists.
- Direct sync into the Codex user skill store remains possible through the existing helper script when that exact destination is required.
