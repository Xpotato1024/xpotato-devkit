---
name: devkit-encoding-hygiene
description: Use when an AI agent needs to detect or normalize UTF-8, BOM, newline, or text-hygiene issues with devkit encoding commands.
---

# devkit encoding hygiene

Use `devkit` to keep text handling deterministic and verifiable.

## When to use

- A file may have mojibake, BOM, control characters, or mixed newlines.
- A documentation or config change should be checked for encoding safety before commit.
- A file or file set should be normalized without manual editor-specific behavior.

## Workflow

1. Check the current state.
   - `devkit encoding check <files> --brief`
   - Use plain output without `--brief` only when per-file issue detail is needed for humans.
2. Normalize only the affected files.
   - `devkit encoding normalize <files> --newline lf`
   - `devkit encoding normalize <files> --newline crlf`
   - Add `--dry-run` first if the scope is uncertain.
3. Re-check after normalization.
   - `devkit encoding check <files> --brief`

## Rules

- Do not normalize broad file sets unless the task actually calls for it.
- Match the repository's newline expectations instead of forcing a personal preference.
- Prefer targeted checks for touched files before broader sweeps.

## Reference

- See [output conventions](../../docs/design/output_conventions.md) for `--brief` expectations.
