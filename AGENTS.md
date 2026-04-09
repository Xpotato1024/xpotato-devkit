# AGENTS.md

## Scope
- This repository contains the `devkit` project, a repo-agnostic CLI toolkit for AI-assisted development workflows.
- Keep the project generic. Do not introduce assumptions that depend on Gale, a specific repository layout, or project-specific artifact names.
- This repository is for the tool itself, not for operating or modifying external target repositories.
- If infrastructure, deployment, release operations, or runtime source of truth lives outside this repository, do not cross that boundary unless the user explicitly asks.

## Product Direction
- `devkit` is a general-purpose CLI for:
  - encoding and text sanity checks
  - diff summarization
  - block extraction and replacement
  - patch application
  - git helper commands
- Prefer reusable building blocks over one-off scripts tied to a single repository.
- Keep the boundary clear:
  - `devkit` handles extraction, replacement, patching, summarization, validation, and deterministic helper workflows
  - external tools or projects may handle search, ranking, repository-specific interpretation, or higher-level orchestration

## Working Environment
- Work from the repository root.
- Use Python with `uv` for dependency and environment management.
- Do not suggest `pip install`; use `uv sync`, `uv add`, and `uv run`.
- Target cross-platform use, with Windows, Linux, and WSL as primary environments.
- Start inspection with deterministic repo state checks:
  - `git status --short --branch`
  - `git diff --stat`
  - `git diff --name-only`
- Read only the files and line ranges needed for the current task.
- Do not open entire long files when focused diff, block extract, or section reads are enough.
- Do not use `Get-Content -Raw`, `cat`, or equivalent whole-file reads for markdown, docs, or source files unless the entire file is required for the task.
- For documentation review, prefer section-scoped reads, diffs, or `devkit` extraction commands over raw whole-file shell reads.
- Encoding normalization commands are allowed only when the task is explicitly about encoding, mojibake, newline, or text-hygiene verification.
- Keep secrets out of the repository. Never commit tokens, keys, passwords, or local machine secrets.
- If you add a new dependency, command, or setup step, document it in `README.md` and, when appropriate, under `docs/`.

## Preferred Skills
When the task matches the workflows below, explicitly use the corresponding Codex skill.

- Use `$devkit-inspect-edit-verify` for scoped code inspection, diff summarization, block extraction, patch diagnosis, patch application, and post-edit verification.
- Use `$devkit-doc-edit` for `README.md`, `docs/`, markdown section edits, implementation notes, benchmark notes, and proposal updates.
- Use `$devkit-encoding-hygiene` for UTF-8, BOM, newline, mojibake, and text-hygiene checks or normalization.
- Use `$devkit-git-drafts` for commit message drafts, PR body drafts, and guarded push workflows.
- Use `$devkit-tree-explore` when repository structure should be surveyed before opening files in detail.
- Use `$devkit-project-bootstrap` for local `devkit` installation checks, local skill sync, or `devkit.toml` initialization.
- Use `$devkit-metrics-review` for reviewing local `devkit` usage metrics.
- Use `$devkit-release-maintainer` for release, tagging, installer, packaging, and release-version verification work.

## Skill Invocation Rule
- If a task clearly matches one of the workflows above, do not improvise first; invoke the matching skill and follow it unless this repository has a stronger local rule.
- If multiple skills apply, use the smallest set that fully covers the task.
- Prefer `--brief`, structured text, or JSON-oriented output when the result will be consumed by another tool or agent.
- If `devkit` is available, prefer it as the first choice for deterministic inspection and reporting over ad-hoc shell usage.
- Do not fall back to broad manual file reading if `devkit` can extract the needed scope safely.
- When working inside the `devkit` repository itself, distinguish clearly between:
  - installed/stable `devkit` for normal repository exploration and completed command usage
  - in-repository development `devkit` for validating current code changes
- Do not assume that `uv run devkit ...` is always the preferred path.
- For completed and stable commands, prefer the installed `devkit` binary when the goal is ordinary inspection, because it reduces warning noise, duplicate output, and token usage.
- Use repository-scoped `uv run devkit ...` only when unpublished local changes must be validated or when the command under test is currently being modified.

## Devkit Availability
- Prefer the installed `devkit` on `PATH` for normal repository exploration and for commands that are already completed and stable.
- If `devkit` is not on `PATH`, use the repository-local or user-local development environment through `uv run`.
- When working inside the `devkit` repository itself, use `uv run devkit ...` only for:
  - validating local unpublished changes
  - testing commands currently under active modification
  - regression checks that must target the in-repository development version
- Do not use `uv run --no-project devkit ...` in normal self-development workflows, because it detaches execution from the repository project context and may add warning noise or invoke an unintended executable path.
- A local Windows development path such as `D:\Xpotato-apps\xpotato-devkit` may exist, but do not assume it is universal unless confirmed by the repository or the user.
- If repo-bundled skills need to be synced to the local Codex store, use the project bootstrap workflow rather than ad-hoc copying.

## Devkit Self-Use Policy
- Distinguish clearly between:
  - installed/stable `devkit` for normal repository exploration
  - in-repository development `devkit` for validating current code changes
- For completed and stable commands, prefer the installed `devkit` binary to reduce noise, duplicate output, startup overhead, and token usage.
- Use `uv run devkit ...` only when:
  - the command being tested is under active modification
  - unpublished local changes must be validated
  - regression checks require the in-repository development version
- Use plain `devkit ...` when the goal is to inspect the repository with already-completed functionality such as stable tree, diff, block, or reporting workflows.
- If command behavior matters, verify which executable path is being used before drawing conclusions from the output.

## Repository Structure
- Keep a clear separation between CLI-facing code and core logic:
  - `src/devkit/commands/` for CLI command definitions, argument parsing, and output rendering
  - `src/devkit/core/` for reusable logic independent of the CLI layer
- Keep tests under `tests/`.
- Keep design notes, requirements, architecture documents, and reports under `docs/`.
- Prefer extending existing modules over adding new top-level files unless a new file clearly improves maintainability.

## Execution Rules
- Read `README.md` and relevant files under `docs/` before changing behavior, interfaces, or structure.
- Keep changes small and focused on one task per branch when possible.
- Prefer incremental implementation over broad speculative abstraction.
- Do not introduce Gale-specific names, fixed paths, artifact schemas, or assumptions into the public CLI unless explicitly requested.
- If a change increases scope beyond the original task, stop expanding and keep the patch limited.
- Do not use repository-scoped source execution uniformly for all `devkit` commands; prefer the shortest stable execution path that matches the purpose.

## Editing Policy
- Prefer minimal diffs over regenerated content.
- Never rewrite an entire file unless explicitly requested.
- Never delete and recreate a long file just to work around insertion, encoding, or editing problems.
- For files longer than 300 lines, edit only the relevant function, section, or block.
- Prefer multiple small patches over one large rewrite.
- If direct insertion is error-prone, switch to a safer patch-based or file-based editing method.
- Preserve existing behavior, formatting, and surrounding structure unless the task explicitly requires broader changes.
- Do not perform opportunistic refactors during a focused task.

## CLI Design Rules
- Keep command names short, consistent, and composable.
- Prefer subcommand structures such as:
  - `devkit encoding ...`
  - `devkit diff ...`
  - `devkit block ...`
  - `devkit patch ...`
  - `devkit git ...`
- Design commands so they can be used both by humans and by coding agents.
- Prefer stdout, file output, and JSON output over interactive-only behavior.
- Prefer deterministic behavior and explicit errors over silent fallback logic.
- Treat user-visible version reporting as release metadata, not just package metadata.
- For tagged releases, ensure user-facing version outputs such as `devkit -V`, installer `--version`, release artifacts, and installer manifests align with the pushed release tag.
- If package version values and release tags intentionally differ, document and verify the release-version injection path instead of assuming package metadata alone is sufficient.

## Verification Policy
- Prefer deterministic local checks over prose reasoning whenever possible.
- If behavior can be verified by a test, command, or script, use that instead of describing expected behavior in prose only.
- Add or update tests when changing behavior that is stable and testable.
- Run targeted checks relevant to the changed area unless broader validation is clearly needed.
- Before committing, review the diff and confirm that unrelated lines were not changed accidentally.
- For text handling changes, verify encoding, newline behavior, and failure modes explicitly.
- For release-related changes, verify both local fallback behavior and tagged-release behavior when version or packaging metadata is affected.

## Output Policy
- Do not print full file contents unless explicitly requested.
- Keep explanations short and factual.
- Prefer reporting:
  1. changed files
  2. one-line purpose per file
  3. checks run
  4. remaining risks
- Do not include long code quotations when a short summary is enough.

## Documentation Policy
- Update `README.md` when user-facing commands, setup, or examples change.
- Update `docs/` when design decisions, architecture, or requirements change materially.
- Keep `docs/` well-organized into hierarchical subdirectories like:
  - `docs/reports/`
  - `docs/proposals/`
  - `docs/design/`
- Every completed implementation task should leave an implementation report under `docs/reports/`.
- Prefer concise, durable documentation over task-specific chatter.
- When adding examples, keep them generic and not tied to a single private repository.

## Git and Review Rules
- Use feature branches for all work.
- Never work directly on `main`.
- Do not push directly to `main`.
- Create commits and Pull Requests as part of the normal workflow.
- Open a Pull Request before merging.
- Do not merge unless explicitly instructed by the user.
- Use short, descriptive branch names. When an agent creates the branch, a `codex/` prefix is acceptable.
- If the user asks to "commit / PR / merge", treat those as separate steps unless merge is explicitly requested.
- After pushing, prefer checking PR status and then stop unless the user explicitly asks for follow-up.
- Write commit messages clearly and consistently. Japanese is acceptable if that is the repository convention.
- If mojibake or terminal encoding issues appear while preparing a commit message or PR body, write the text to a temporary UTF-8 file first and submit it from that file.

## Quality Bar
- Favor correctness, composability, and maintainability over cleverness.
- Keep the first implementation minimal, then generalize only when real usage justifies it.
- Avoid overengineering configuration, plugin systems, or abstraction layers in the initial phase.
- If performance becomes a real bottleneck, document it and isolate the affected core logic so future Rust migration remains possible.

## Safety Checks
- Never overwrite user changes unless explicitly asked.
- Never commit generated secrets, caches, or local machine state.
- If a command may alter history or remove files, verify the target carefully before running it.
- For destructive operations, prefer dry-run support or an explicit confirmation path when practical.
