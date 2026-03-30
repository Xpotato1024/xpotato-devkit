# xpotato-devkit

`xpotato-devkit` is a repo-agnostic CLI toolkit for AI-assisted development workflows.
It provides deterministic utilities for checking encoding, extracting diffs, manipulating text blocks, and generating AI prompts, designed to minimize token usage and enhance safety.

## Features

- **`devkit encoding check [files...]`**: Verify text files for UTF-8 validity, BOM, replacement characters, and mixed newlines to prevent AI hallucinations.
- **`devkit diff summarize`**: Generate brief diff statistics (`added`, `deleted` lines) for the current workspace.
- **`devkit block extract / replace`**: Dynamically extract or replace specific lines, Markdown headings, or functions to keep AI payloads small.
- **`devkit git commit-message / pr-body`**: Automatically generate context-rich templates containing diff stats to feed into AI assistants (like Cursor/Gale) for high-quality commit and PR drafts.
- **`devkit git safe-push`**: Standardized pushing command with safe-guards against direct pushes to `main` / `master`.

## Installation

This project is built for use with [uv](https://github.com/astral-sh/uv).

```bash
uv sync
```

To run commands, use `uv run`:

```bash
uv run devkit --help
uv run devkit encoding check "*.py"
```

## Configuration

You can customize the toolkit behavior locally using a `devkit.toml` file in the root directory.

```toml
[encoding]
# Ignore paths from encoding checks
ignore = [".git", "node_modules", "dist", ".venv", "__pycache__"]

[git]
# Default language for AI instructions in commit/PR drafts
lang = "ja"  # Use "en" for English
```

## License

MIT License
