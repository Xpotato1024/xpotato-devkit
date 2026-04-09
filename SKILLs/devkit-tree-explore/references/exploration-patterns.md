# Exploration Patterns

Use this reference when `devkit tree` should drive repository orientation.

## Narrow-first patterns

- Small subtree:
  - `devkit tree --path <dir> --max-depth 2`
- Directory-only scan:
  - `devkit tree --path <dir> --dirs-only`
- Extension-focused scan:
  - `devkit tree --path <dir> --ext .rs`

## Follow-up pattern

1. `devkit tree`
2. Pick likely files or directories
3. Use:
   - `devkit block outline`
   - `devkit block extract`
   - `devkit diff summarize`

## Reporting rule

- Use tree output to compress structure quickly.
- Do not treat tree output alone as sufficient verification of file contents.
