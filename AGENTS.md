# AGENTS.md

## Scope
- This repository contains the `devkit` project, a repo-agnostic CLI toolkit for AI-assisted development workflows.
- Keep the project generic. Do not introduce assumptions that depend on Gale, a specific repository layout, or project-specific artifact names.
- This repository is for the tool itself, not for operating or modifying external target repositories.

## Product Direction
- `devkit` is a general-purpose CLI for:
  - encoding and text sanity checks
  - diff summarization
  - block extraction and replacement
  - patch application
  - git helper commands
- Prefer reusable building blocks over one-off scripts tied to a single repository.
- Keep the boundary clear:
  - `devkit` handles extraction, replacement, patching, summarization, and validation
  - external tools or projects may handle search, ranking, or project-specific interpretation

## Working Environment
- Use Python with `uv` for dependency and environment management.
- Do not suggest `pip install`; use `uv sync`, `uv add`, and `uv run`.
- Target cross-platform use, with Windows, Linux, and WSL as primary environments.
- Keep secrets out of the repository. Never commit tokens, keys, passwords, or local machine secrets.
- If you add a new dependency, command, or setup step, document it in `README.md` and, when appropriate, under `docs/`.

## Repository Structure
- Keep a clear separation between CLI-facing code and core logic:
  - `src/devkit/commands/` for CLI command definitions, argument parsing, and output rendering
  - `src/devkit/core/` for reusable logic independent of the CLI layer
- Keep tests under `tests/`.
- Keep design notes, requirements, and architecture documents under `docs/`.
- Prefer extending existing modules over adding new top-level files unless a new file clearly improves maintainability.

## Execution Rules
- Read `README.md` and relevant files under `docs/` before changing behavior, interfaces, or structure.
- Keep changes small and focused on one task per branch when possible.
- Prefer incremental implementation over broad speculative abstraction.
- Do not introduce Gale-specific names, fixed paths, artifact schemas, or assumptions into the public CLI unless explicitly requested.
- If a change increases scope beyond the original task, stop expanding and keep the patch limited.

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

## Verification Policy
- Prefer deterministic local checks over prose reasoning whenever possible.
- If behavior can be verified by a test, command, or script, use that instead of describing expected behavior in prose only.
- Add or update tests when changing behavior that is stable and testable.
- Run targeted checks relevant to the changed area unless broader validation is clearly needed.
- Before committing, review the diff and confirm that unrelated lines were not changed accidentally.
- For text handling changes, verify encoding, newline behavior, and failure modes explicitly.

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
- Keep `docs/` well-organized into hierarchical subdirectories like `docs/reports/`, `docs/proposals/`, and `docs/design/`.
- **Implementation Reports**: Every time an implementation task is completed, create a report artifact (in English or Japanese) summarizing the implementation details and place it under `docs/reports/`.
- Prefer concise, durable documentation over task-specific chatter.
- When adding examples, keep them generic and not tied to a single private repository.

## Git and Review Rules
- Use feature branches for all work.
- Never work directly on `main`.
- Do not push directly to `main`.
- Open a Pull Request before merging.
- Do not merge unless explicitly instructed by the user.
- Use short, descriptive branch names. When an agent creates the branch, a `codex/` prefix is acceptable.
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
