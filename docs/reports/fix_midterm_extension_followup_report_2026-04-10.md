# Fix Midterm Extension Follow-up Report

## Changed files

- `rust/crates/devkit-search/Cargo.toml`
  - Added a dedicated Rust crate for repo-local text and symbol search.
- `rust/crates/devkit-search/src/lib.rs`
  - Implemented `search text` / `search symbol`, type and glob filters, context capture, JSON-ready payloads, and unit tests.
- `rust/crates/devkit-block/src/lib.rs`
  - Reused declaration heuristics through `list_functions_in_text` and fixed `block extract --lines` so out-of-range end values no longer panic.
- `rust/crates/devkit-patch/Cargo.toml`
  - Enabled serde derives for structured patch diagnostics.
- `rust/crates/devkit-patch/src/lib.rs`
  - Added structured patch classifications, file/hunk diagnostics, recommended next steps, and stable invalid-patch brief output.
- `rust/crates/devkit-cli/Cargo.toml`
  - Wired the CLI to the new `devkit-search` crate.
- `rust/crates/devkit-cli/src/main.rs`
  - Added `devkit search text` / `devkit search symbol`, search view validation and rendering, richer patch JSON, and command-surface regression coverage.
- `rust/crates/devkit-cli/tests/cli_smoke.rs`
  - Added integration smoke tests that execute `tree`, `encoding`, `block`, `diff`, `search`, `patch diagnose`, `doc`, `git`, and `metrics` against a temp repo.
- `README.md`
  - Promoted `search` into the main workflow and command list.
- `docs/design/ai_agent_workflow.md`
  - Updated the AI workflow to use implemented `search` commands and stronger patch diagnosis guidance.
- `docs/design/command-gap-analysis.md`
  - Replaced the old docs-only search status with implemented behavior and narrowed the remaining gaps.
- `docs/concepts/why-devkit.md`
  - Expanded the `patch diagnose` explanation around concrete failure classes and next actions.
- `docs/reports/fix_midterm_extension_phase1_report_2026-04-10.md`
  - Marked the earlier report as a phase-1 snapshot so it no longer reads as the latest branch state.

## Checks run

- `cargo test -p devkit-search -- --nocapture`
- `cargo test -p devkit-patch -- --nocapture`
- `cargo test -p devkit-block -- --nocapture`
- `cargo test -p devkit-cli -- --nocapture`
- `cargo test -p devkit-cli --test cli_smoke -- --nocapture`
- `cargo build -p devkit-cli`

## Remaining risks

- `search symbol` is declaration search only. Reference search and semantic search remain out of scope.
- The command audit now covers runtime smoke and help execution locally, but release asset publication still depends on a real tag-triggered GitHub Actions run.
- A stale higher-priority `devkit.exe` on PATH can still mask a fresh install until the installed binary is refreshed or PATH order is corrected.
