# architecture

## Layering

- `devkit.commands`: CLI entrypoints, argument parsing, rendering
- `devkit.core`: reusable logic without CLI concerns

## Initial command groups

- encoding
- diff
- block
- patch
- git
- bootstrap

## Bootstrap placement

- The canonical self-install/bootstrap entrypoint should be a `devkit` CLI command so it works the same way on Windows, Linux, and macOS.
- Repository-local wrapper scripts may exist for compatibility, but they should delegate to shared Python code instead of carrying separate logic.
- Avoid making bootstrap depend on shell-specific task runners by default; prefer `uv run devkit ...` as the cross-platform path.
