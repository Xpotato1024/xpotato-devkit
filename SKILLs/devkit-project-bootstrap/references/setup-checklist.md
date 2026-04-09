# Setup Checklist

Use this reference when walking through local installation or initial config.

## Installation flow

1. Confirm whether `devkit` is already resolvable on PATH.
2. Run:
   - `devkit bootstrap install-self`
3. Verify:
   - `devkit --help`

## Config initialization flow

1. Confirm whether `devkit.toml` already exists.
2. Run:
   - `devkit config init`
3. Verify:
   - `devkit encoding check devkit.toml --brief`

## Guardrails

- Do not overwrite an existing config without explicit approval.
- Prefer the CLI workflow over manual file creation.
