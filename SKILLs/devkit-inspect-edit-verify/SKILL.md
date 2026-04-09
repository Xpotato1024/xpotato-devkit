---
name: devkit-inspect-edit-verify
description: Use when an AI agent needs to inspect diffs or code blocks, edit locally, and verify the result with devkit's diff, block, and patch commands.
---

# devkit inspect-edit-verify

Use `devkit` to keep the edit loop short and deterministic.

## When to use

- A change needs a diff summary before editing.
- A file is too large to read end-to-end.
- A patch should be checked before application.
- The result needs a factual verification pass.

## Workflow

1. Inspect the change surface.
   - `devkit diff summarize`
   - `devkit diff summarize --json`
   - `devkit block outline <file>`
   - `devkit block context <file> <function>`
   - `devkit block extract <file> ...`
2. Edit the smallest possible block.
   - Prefer block selectors, headings, markers, or line ranges over full-file rewrites.
   - Use `devkit patch diagnose <patch-file>` before `devkit patch apply <patch-file>`.
3. Verify the result.
   - Re-run `devkit diff summarize`
   - Re-run the block command that identified the edit target
   - Use `--brief` for one-line status and `--json` when another tool will consume the output

## Rules

- Do not guess at file state when `patch diagnose` fails.
- Do not fall back to whole-file reads unless the selector cannot be resolved.
- Keep repo-specific interpretation outside `devkit`; use it for extraction, patching, and factual reporting.
- Prefer `--brief` or JSON output for agent-to-agent/tool-to-tool handoff because the default human-facing terminal output may be colorized.

## Reference

- See [AI agent workflow](../../docs/design/ai_agent_workflow.md) for the full inspect/edit/verify sequence.
- When choosing between headings, symbols, context, and line selectors, read [references/selector-strategy.md](references/selector-strategy.md).
