# xpotato-devkit

[English](README.md) | [日本語](README.ja.md)

[![Rust Workflow](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml/badge.svg)](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`devkit` is a repo-agnostic CLI for deterministic AI-assisted development workflows.
The actively maintained implementation is the Rust-native CLI. It focuses on reliable inspection, extraction, replacement, validation, and Git helper flows that work well for both humans and coding agents.

> [!NOTE]
> The legacy Python implementation remains available for reference, but new functionality is added to the Rust CLI.

## What It Is

`devkit` is designed around a repeatable workflow:

1. inspect only the scope you need
2. localize the relevant structure or context
3. apply or verify a focused change

The toolkit currently emphasizes:

- encoding and text sanity checks
- diff summarization
- text and symbol search
- block extraction and replacement
- patch diagnosis and application
- markdown section editing
- Git drafting helpers

For the rationale behind the tool, see [Why devkit](docs/concepts/why-devkit.md).

## What It Helps Prevent

`devkit` is intended to reduce common failure modes such as:

- reading entire repositories when only a small scope is needed
- relying on fragile ad-hoc shell pipelines for structured inspection
- applying patches before checking whether they already match the target
- overusing noisy Windows shell commands such as `Get-Content` or recursive directory enumeration
- mixing human-facing and agent-facing output in ways that are hard to parse

## Who It Is For

- AI-agent workflows that prefer `--brief` or structured output
- developers who want narrow, deterministic inspection instead of broad file reads
- Windows and PowerShell users who want a safer alternative to repetitive one-off shell commands

## 3-Minute Tour

```bash
# 1. Inspect the overall change surface
devkit diff summarize --name-status --limit 20

# 2. Search by text or symbol
devkit search text "patch diagnose" --glob "*.md" --context 2
devkit search symbol PatchDiagnostic --type rust

# 3. Localize file structure and nearby context
devkit block outline rust/crates/devkit-cli/src/main.rs
devkit block context rust/crates/devkit-cli/src/main.rs format_timing_output

# 4. Diagnose a patch before applying it
devkit patch diagnose --patch-file .tmp/change.diff

# 5. Apply the patch if the diagnosis is clean
devkit patch apply --patch-file .tmp/change.diff
```

See [AI agent workflow](docs/design/ai_agent_workflow.md) for the intended inspect/edit/verify loop.

## Main Command Groups

The current Rust binary publicly supports these command groups:

- `devkit encoding check`
- `devkit encoding normalize`
- `devkit tree`
- `devkit block outline`
- `devkit block context`
- `devkit block extract`
- `devkit block replace`
- `devkit bootstrap sync-skills`
- `devkit bootstrap init-agents`
- `devkit md append-section`
- `devkit md replace-section`
- `devkit md ensure-section`
- `devkit md append-bullet`
- `devkit diff summarize`
- `devkit search text`
- `devkit search symbol`
- `devkit patch diagnose`
- `devkit patch apply`
- `devkit bootstrap install-self`
- `devkit config init`
- `devkit doc impl-note`
- `devkit doc benchmark-note`
- `devkit git commit-message`
- `devkit git pr-body`
- `devkit git safe-push`
- `devkit metrics show`

Implemented global flags:

- `--brief`
- `--time`
- `--time-json`

Human-readable terminal output may use ANSI color where supported. `--brief` and JSON-oriented output remain plain text for scripts and agents.

## Installation

### 1. Install From Cargo

If you have a Rust toolchain, installing from source is the fastest path to the latest CLI:

```bash
cargo install --git https://github.com/Xpotato1024/xpotato-devkit --branch main devkit-cli
```

If you already have a local checkout:

```bash
cargo install --path rust/crates/devkit-cli
```

If `devkit --help` is missing commands such as `encoding` or `search`, or if `devkit block extract --help` is missing `--list-headings`, the first `devkit.exe` on `PATH` may be stale. Reinstall from the current checkout and check resolution order with `where devkit` on Windows.

```bash
cargo install --path rust/crates/devkit-cli --force
```

### 2. Download Release Assets

If you do not have a Rust environment, use the published assets from [GitHub Releases](https://github.com/Xpotato1024/xpotato-devkit/releases):

- Windows: `devkit-installer-<tag>-x86_64-pc-windows-msvc.exe` or the zip asset
- Linux / macOS: `devkit-<tag>-<target>.tar.gz`

For the Windows installer contract and `winget` preparation notes, see [Windows winget prep](docs/install/windows-winget-prep.md).

## Common Examples

```bash
# Show help
devkit --help

# Summarize changed files
devkit diff summarize --name-status --limit 20

# Search text or symbols
devkit search text "safe-push" --glob "*.md" --context 2
devkit search symbol PatchDiagnostic --type rust

# Inspect shallow directory structure
devkit tree --max-depth 2 --limit 40

# Check UTF-8/BOM/newline issues
devkit encoding check README.md docs/**/*.md --brief

# Sync repo-bundled skills to another workspace
devkit bootstrap sync-skills --repo-root . --target ../target-repo

# Generate an AGENTS.md starter for another workspace
devkit bootstrap init-agents --path ../target-repo/AGENTS.md

# Normalize BOM and newlines
devkit encoding normalize README.md docs/**/*.md --newline crlf

# Show Python file structure
devkit block outline main.py --imports

# Draft a commit message from staged changes
devkit git commit-message --staged

# Push safely when upstream may not exist
devkit git safe-push --remote origin --yes
```

## Legacy / Planned Scope

The broader project direction and the legacy Python-era capability set are documented here even when the current Rust binary has a narrower public surface.

- `devkit encoding check [files...]`
- `devkit diff summarize`
- `devkit search text / search symbol`
- `devkit block extract / replace`
- `devkit block outline / context`
- `devkit tree`
- `devkit md`
- `devkit doc`
- `devkit git`
- `devkit patch apply / diagnose`

## Python Legacy

The old Python implementation lives under `legacy/python/` for reference and compatibility work.
If you still need that environment:

```bash
cd legacy/python
uv sync
uv run devkit-py --help
```

See [Python Legacy policy](docs/legacy/python-legacy.md) for details.

## Configuration (`devkit.toml`)

`devkit` can load project-local defaults from `devkit.toml`. If you need an explicit config path, set `DEVKIT_CONFIG`. To create a starter config file, run `devkit config init`.

```toml
[encoding]
ignore = [".git", "node_modules", "dist", ".venv", "__pycache__"]

[git]
lang = "ja"
```

## Rust Verification

Before pushing a Rust CLI change, run the relevant local checks:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p devkit-block -p devkit-core -p devkit-metrics -p devkit-patch -p devkit-cli
```

## AI Workflow

For agent-facing guidance, see:

- [AI agent workflow](docs/design/ai_agent_workflow.md)
- [Why devkit](docs/concepts/why-devkit.md)
- [Command gap analysis](docs/design/command-gap-analysis.md)
- `SKILLs/devkit-doc-edit/`
- `SKILLs/devkit-encoding-hygiene/`
- `SKILLs/devkit-git-drafts/`
- `SKILLs/devkit-inspect-edit-verify/`
- `SKILLs/devkit-metrics-review/`
- `SKILLs/devkit-project-bootstrap/`
- `SKILLs/devkit-release-maintainer/`
- `SKILLs/devkit-tree-explore/`

Bundled repo skills should prefer `--brief` or JSON-capable output when another tool or agent will consume the result.

## Release

Release is tag-triggered only:

- Push a `v*` tag to build and publish Windows, Linux, and macOS Apple Silicon assets.
- Pushes to `main` and pull requests do not create releases.
- Windows releases publish both `devkit-{tag}-x86_64-pc-windows-msvc.zip` and `devkit-installer-{tag}-x86_64-pc-windows-msvc.exe`.
- Release outputs also include `devkit-{tag}-sha256.txt` for checksum verification.

Release note:

- `v0.1.6` is the checked release target for the first `winget` submission. The published Windows installer asset, checksum file, draft manifests, and silent install/uninstall verification are now aligned.

See [tag-triggered release docs](docs/release/tag-triggered-release.md) for the maintainer flow.

## Windows Installation

Windows users can follow [docs/install/windows-installation.md](docs/install/windows-installation.md) for the native user-local install flow.
Linux and macOS release archives ship the `devkit` binary only and do not include a native installer.

The recommended Windows asset is `devkit-installer-{tag}-x86_64-pc-windows-msvc.exe`. By default it installs to `%LOCALAPPDATA%\\Xpotato\\devkit` and adds that directory to the user `PATH`. Use `--unpack-only` to skip `PATH` changes:

```powershell
.\devkit-installer.exe --unpack-only
```

For unattended installation, use `--silent`. `winget` preparation notes and draft manifests live under [docs/install/windows-winget-prep.md](docs/install/windows-winget-prep.md) and `packaging/winget/`.

## License

This project is distributed under the [MIT License](LICENSE).
