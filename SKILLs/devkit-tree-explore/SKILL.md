---
name: devkit-tree-explore
description: Use when an AI agent needs to survey repository structure quickly with devkit tree before reading files in detail.
---

# devkit tree explore

Use `devkit tree` to compress repository structure before deeper inspection.

## When to use

- The repository or subtree is unfamiliar.
- A user asks where code or docs likely live.
- You need a compact structure view before choosing specific files to open.

## Workflow

1. Start with a narrow tree when possible.
   - `devkit tree`
   - `devkit tree --path <dir>`
   - `devkit tree --max-depth <n>`
2. Filter when the target file type or shape is known.
   - `devkit tree --ext .rs`
   - `devkit tree --dirs-only`
3. Follow with focused reads.
   - `devkit block outline`
   - `devkit block extract`
   - `devkit diff summarize`

## Rules

- Prefer smaller subtrees and bounded depth before scanning the entire repo.
- Use `tree` for orientation, not as a substitute for file-level verification.
- Combine with `block` commands once the likely target files are identified.
