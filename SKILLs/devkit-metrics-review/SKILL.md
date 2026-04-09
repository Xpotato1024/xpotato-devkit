---
name: devkit-metrics-review
description: Use when an AI agent needs to inspect local devkit usage metrics and summarize command frequency, timing, or success patterns.
---

# devkit metrics review

Use `devkit` when the task is about measuring local usage patterns.

## When to use

- A user wants to inspect command frequency or average execution time.
- A workflow change should be validated against local `devkit` usage metrics.
- A metrics file may be missing, empty, or disabled and needs deterministic handling.

## Workflow

1. Check whether metrics are configured.
   - `devkit metrics show`
   - If needed, inspect `devkit.toml`
2. Summarize the relevant signals.
   - Command count
   - Average run time
   - Brief usage rate
   - Success rate
3. Report gaps explicitly.
   - Metrics disabled
   - File missing
   - No valid records

## Rules

- Treat `metrics show` output as observational, not causal proof.
- Preserve the distinction between missing data and healthy low activity.
- Prefer quoting the actual command labels shown by `devkit metrics show`.
