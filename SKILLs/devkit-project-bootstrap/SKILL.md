---
name: devkit-project-bootstrap
description: Use when an AI agent needs to install devkit into a local environment or initialize devkit project configuration with bootstrap and config commands.
---

# devkit project bootstrap

Use `devkit` when the task is about initial local setup.

## When to use

- A checkout should be installed as a local user tool.
- A repository needs an initial `devkit.toml`.
- A user wants the smallest deterministic setup path instead of manual steps.

## Workflow

1. Inspect the current environment.
   - Confirm whether `devkit` is already on PATH.
   - Confirm whether `devkit.toml` already exists.
2. Run the narrow setup step.
   - `devkit bootstrap install-self`
   - `devkit config init`
3. Verify the result.
   - `devkit --help`
   - `devkit config init --help` if the config path or overwrite behavior matters
   - `devkit encoding check devkit.toml --brief` when a config file was written

## Rules

- Do not overwrite an existing config unless the task explicitly allows it.
- Keep setup instructions generic and cross-platform.
- Prefer the CLI flow over ad hoc manual file creation.
