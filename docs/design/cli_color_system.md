# CLI Color System

## Purpose

This document defines a consistent color system for human-facing `devkit` CLI output.

Goals:

- Make important information easier to scan in terminals.
- Keep command semantics consistent across command groups.
- Preserve deterministic agent and script workflows by keeping `--brief` and JSON output uncolored.
- Keep the design generic and reusable across repositories and operating systems.

## Scope

This color system applies to standard human-facing terminal output from the Rust CLI.

Out of scope:

- `--brief` output
- JSON output
- stderr timing output
- repository-specific theming

## Design Principles

1. Semantic first
Color should communicate meaning, not decorate arbitrary text.

2. Plain-text fallback
When color is unsupported or disabled, output must remain equally understandable as plain text.

3. Stable status language
The same status concept should use the same color across commands.

4. Moderate saturation
Use readable, restrained terminal colors rather than overly bright rainbow output.

5. Accessibility
Do not rely on color alone. Status words such as `OK`, `FAIL`, `CHANGED`, and headings must remain explicit in text.

## Core Semantic Palette

These names describe meaning, not a required ANSI implementation detail.

- `header`
  - Purpose: section titles, table headers, command summaries
  - Suggested tone: cyan or bright cyan
- `ok`
  - Purpose: success states, completed actions, healthy totals
  - Suggested tone: green
- `warn`
  - Purpose: cautions, non-fatal anomalies, would-change states, binary markers
  - Suggested tone: yellow
- `fail`
  - Purpose: failures, blocking issues, invalid states
  - Suggested tone: red
- `info`
  - Purpose: counts, secondary summary values, informative metrics
  - Suggested tone: light cyan or blue-cyan
- `path`
  - Purpose: file paths, command names in tables, user-targeted references
  - Suggested tone: soft blue
- `dir`
  - Purpose: directory names in tree-style output
  - Suggested tone: blue
- `added`
  - Purpose: additions, positive numeric deltas
  - Suggested tone: green
- `deleted`
  - Purpose: deletions, negative numeric deltas
  - Suggested tone: red
- `dim`
  - Purpose: separators, file sizes, low-priority summary text
  - Suggested tone: dim or gray
- `muted`
  - Purpose: unchanged states, low-emphasis labels
  - Suggested tone: gray

## Command-Level Rules

### 1. Tree output

- Directories use `dir`.
- File names use neutral foreground or `path`.
- File sizes use `dim`.
- Final tree summary uses `dim`.

### 2. Encoding output

- `OK` uses `ok`.
- `FAIL` uses `fail`.
- File paths use `path`.
- Issue labels use `warn`.
- Summary rows use:
  - `header` for the label
  - `info` for checked-file counts
  - `ok` or `warn` for issue counts depending on whether issues exist

### 3. Normalize output

- `CHANGED` uses `ok`.
- `WOULD_CHANGE` uses `warn`.
- `UNCHANGED` uses `muted`.
- File paths use `path`.

### 4. Diff summarize output

- Section title uses `header`.
- Separator rules use `dim`.
- Table headings use `header`.
- File paths use `path`.
- Addition counts use `added`.
- Deletion counts use `deleted`.
- Binary markers use `warn`.
- Total row uses `header` plus `added` and `deleted`.

### 5. Metrics output

- Main title uses `header`.
- Table headings use `header`.
- Command names use `path`.
- Run counts use `info`.
- Average durations use neutral foreground.
- Success percentages use:
  - `ok` for healthy results
  - `warn` for degraded results
- Brief percentages use `warn` because they are informational, not success/failure states.

### 6. Installer output

Installer output should follow the same semantic system if color is introduced later:

- success steps: `ok`
- warnings and PATH conflicts: `warn`
- blocking errors: `fail`
- install paths and binary paths: `path`
- section labels such as `PATH status` and `Payload source`: `header`

## Behavioral Rules

### Color enablement

Color may be enabled only when output is intended for a terminal-oriented human reader.

Recommended conditions:

- stdout is a terminal, or color is explicitly forced
- `NO_COLOR` is not set
- `CLICOLOR=0` is not set
- the command is not in `--brief` mode
- the command is not emitting JSON

### Environment compatibility

Support these conventions where practical:

- `NO_COLOR`
- `CLICOLOR`
- `CLICOLOR_FORCE`
- `FORCE_COLOR`

### Fallback rules

- All colored output must remain readable when escape codes are stripped.
- Alignment-sensitive tables should preserve readable spacing with and without color.

## Non-Goals

- Per-command custom themes
- Repository branding colors
- Syntax highlighting for arbitrary file contents
- Full editor-like rendering inside the terminal

## Future Extensions

Possible later additions:

- shared output styling helpers for installer and future binaries
- colorized warning/error helpers on stderr
- optional richer code-block rendering for commands that intentionally display source excerpts
- snapshot tests that validate plain-text fallbacks and ANSI-enabled output separately

## Relationship To Other Docs

- `docs/design/output_conventions.md` defines output invariants such as `--brief`, JSON, timing, and ANSI policy.
- This document defines the semantic color language applied on top of those invariants.
