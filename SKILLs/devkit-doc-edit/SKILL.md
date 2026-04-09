---
name: devkit-doc-edit
description: Use when an AI agent needs to edit README or docs with devkit markdown and documentation helpers, especially for implementation notes, benchmark notes, and section-safe updates.
---

# devkit doc edit

Use `devkit` when the task is primarily about documentation changes.

## When to use

- A README or `docs/` file needs a targeted section update.
- A report, implementation note, or benchmark note should be generated or refreshed.
- A markdown heading, section body, or bullet list should be modified without rewriting the whole file.

## Workflow

1. Inspect the target document.
   - `devkit encoding check <files> --brief`
   - `devkit block extract <file> --heading <heading>`
   - `devkit block extract <file> --list-headings`
2. Apply the narrowest markdown operation.
   - `devkit md append-section`
   - `devkit md replace-section`
   - `devkit md ensure-section`
   - `devkit md append-bullet`
3. Generate draft note content when needed.
   - `devkit doc impl-note`
   - `devkit doc benchmark-note`
4. Verify the result.
   - Re-run `devkit encoding check <files> --brief`
   - Re-run the relevant `block extract` command for the touched section

## Rules

- Prefer section-scoped edits over full-file rewrites.
- Keep examples generic and repo-agnostic unless the repository explicitly documents project-local behavior.
- Use `--brief` when another tool or agent will consume the result.

## Reference

- See [AI agent workflow](../../docs/design/ai_agent_workflow.md) for the inspect/edit/verify loop.
