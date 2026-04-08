# xpotato-devkit

[![Rust Workflow](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml/badge.svg)](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

xpotato-devkit は、AI支援開発のための repo-agnostic な CLI ツールキットです。
現在の主実装は **Rust 製ネイティブCLI** であり、高速性・配布性・安定した実行を重視しています（[ベンチマーク詳細](docs/benchmarks/python-vs-rust.md) によれば、Python版と比較して起動速度が約12倍、抽出速度が最大20倍向上しています）。差分集計、ブロック抽出・置換、AI向けテンプレート生成補助などツールとして確定的（deterministic）なユーティリティを提供し、LLM使用時のトークン消費量を最小限に抑えつつ、安全性を高めることを目的としています。

> [!NOTE]
> Python版は後方互換のために同梱されていますが、現行の推奨実装および新機能追加の対象は Rust版 のみです。

## Current Rust CLI Scope

The current Rust binary publicly supports these command groups:

- `devkit encoding check`
- `devkit encoding normalize`
- `devkit tree`
- `devkit block outline`
- `devkit block context`
- `devkit block extract`
- `devkit block replace`
- `devkit md append-section`
- `devkit md replace-section`
- `devkit md ensure-section`
- `devkit md append-bullet`
- `devkit diff summarize`
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

Global flags currently implemented in Rust:

- `--brief`
- `--time`
- `--time-json`

> [!IMPORTANT]
> The Rust CLI now covers the primary public command groups described in this repository.
> Older design notes may still mention exploratory variants that are not part of the stable public CLI surface.

## Legacy / Planned Scope

The items below reflect the broader project direction and legacy Python-era capability set, not the exact command surface of the current Rust binary.

- **`devkit encoding check [files...]`**: テキストファイルの UTF-8 妥当性、BOMの有無、置換文字や混在する改行コードを検査し、AIのハルシネーションを防止。
- **`devkit diff summarize`**: カレントワークスペース、ステージ済み変更、または特定の比較範囲（`--base/--head`）の差分を要約・集計。ファイルパスのみの出力（`--files-only`）も可能。
- **`devkit block extract / replace`**: 長大なファイルから行数や見出し、関数定義等を基準に特定のブロックだけを抽出・置換。Rust/Go/Python/C/JS など各言語の構造解析に対応し、`--symbol` 指定や `--list-headings` などの探索補助機能あり。
- **`devkit block outline / context`**: 
  - `outline`: ファイル中の関数/クラスのシグネチャのみを行番号付きで抽出し構造を把握。トークンを大幅に節約。
  - `context`: 指定シンボルの前後数行のコンテキストを行番号付きで表示し、パッチ作成の文脈として提供。
- **`devkit tree`**: プロジェクトのディレクトリ構造をコンパクトに表示。`.gitignore` や `devkit.toml` の設定を尊重し、不要な出力をカット。
- **`devkit md` 系**: Markdownの特定セクションに対する追記・置換・新規作成（`append-section`, `replace-section`, `ensure-section` 等）。
- **`devkit doc` 系**: 実装メモやベンチマーク記録のテンプレートを自動生成。
- **`devkit git` 系**: コミットメッセージ・PRテンプレート生成、安全なプッシュラッパー (`safe-push`) など。
- **`devkit patch apply / diagnose`**: パッチ適用と診断（hunk 単位の適用可能性判定）。

> **`--brief` モード**: 主要コマンドには `--brief` フラグがあり、出力を `OK: ...` / `FAIL: ...` の 1 行に制限することで、AIエージェントのトークン消費をさらに節約できます。

## インストール方法

### 1. Cargo を使ったインストール
Rust 開発環境がある場合、リポジトリから直接ビルド・インストールするのが最も最新です。

```bash
cargo install --git https://github.com/Xpotato1024/xpotato-devkit --branch main devkit-cli
```
※ローカルクローン済みの場合は `cargo install --path rust/crates/devkit-cli`

### 2. バイナリのダウンロード (GitHub Releases)
Rust環境がない場合は、[GitHub Releases](https://github.com/Xpotato1024/xpotato-devkit/releases) から各OS（Windows, Linux, macOS）用のコンパイル済み実行ファイルをダウンロードし、PATHの通ったディレクトリに配置してください。

> [!TIP]
> **運用フロー:** GitHub 上で `v*`（例: `v0.1.0`）のタグを作成・Pushすると、GitHub Actions が自動的に Windows / Linux / macOS 向けの実行バイナリをビルドし、[Releases](https://github.com/Xpotato1024/xpotato-devkit/releases) に公開・配布する仕組みとなっています。

## よく使うコマンド例

各機能は `devkit` コマンドを通じて実行します。

```bash
# ヘルプと使用可能なコマンド一覧を表示
devkit --help

# UTF-8/BOM/改行混在などの異常を要約チェック
devkit encoding check README.md docs/**/*.md --brief

# BOM 除去と改行コード統一
devkit encoding normalize README.md docs/**/*.md --newline crlf

# pythonファイルの構造（関数・クラスシグネチャ）のみツリー表示
devkit block outline main.py --imports

# コミット文の下書きを生成
devkit git commit-message --staged

# 現在の checkout を cargo install でユーザー環境へインストール
devkit bootstrap install-self

# カレントプロジェクト向けの devkit.toml テンプレートを生成
devkit config init

# metrics が有効な場合にローカル集計を表示
devkit metrics show

# 安全な push。upstream が無い場合は remote を明示
devkit git safe-push --remote origin --yes
```

## Python legacy 版について

以前の Python 版は後方互換性のため [legacy/python/](legacy/python/) に退避されています。
新規機能の追加は行われず、重大なバグ修正のみの保守対象となります。
特殊な環境等でどうしてもPython版が必要な場合は、[uv](https://github.com/astral-sh/uv) 経由でインストールし `devkit-py` プレフィックスを使用します。

```bash
cd legacy/python
uv sync
uv run devkit-py --help
```

詳しくは [Python Legacy ポリシー](docs/legacy/python-legacy.md) をご参照ください。

## 設定ファイル (`devkit.toml`)

既定では、`devkit` は実行時のカレントディレクトリから親方向へ `devkit.toml` を探索し、最初に見つかったものを読み込みます。
別の設定ファイルを明示したい場合は `DEVKIT_CONFIG` 環境変数でパスを指定できます。
雛形が必要な場合は `devkit config init` で生成できます。

```toml
[encoding]
# 検査対象から除外するディレクトリ・ファイル名
ignore = [".git", "node_modules", "dist", ".venv", "__pycache__"]

[git]
# AI連携機能におけるプロンプト言語
lang = "ja"  # 英語の場合は "en" を指定
```

## Rust verification

Rust CLI を変更した PR では、少なくとも次をローカルで通してから push してください。

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p devkit-block -p devkit-core -p devkit-metrics -p devkit-patch -p devkit-cli
```

`fmt` だけでは CI の `Lint & Format` を再現できません。`clippy` は warning を含めて必須です。

## ライセンス

本ソフトウェアは [MIT License](LICENSE) のもとで公開されています。

## AI workflow

For agent-facing guidance, see:

- [AI agent workflow](docs/design/ai_agent_workflow.md)
- `SKILLs/devkit-inspect-edit-verify/`
- `SKILLs/devkit-git-drafts/`

## Release

Release is tag-triggered only:

- Push a `v*` tag to build and publish Windows, Linux, and macOS Apple Silicon assets.
- Pushes to `main` and pull requests do not create releases.
- Release asset names follow `devkit-{tag}-{target}{ext}`.

See [tag-triggered release docs](docs/release/tag-triggered-release.md) for the maintainer flow.

## Windows installation

Windows users can follow [docs/install/windows-installation.md](docs/install/windows-installation.md) for the native user-local install flow.
Linux and macOS release archives ship the `devkit` binary only and do not include a native installer.

The Windows release archive includes `devkit-installer.exe`. Extract it and run the installer. By default it installs to `%LOCALAPPDATA%\\Xpotato\\devkit` and adds that directory to the user PATH. Use `--unpack-only` to skip PATH changes:

```powershell
.\devkit-installer.exe --unpack-only
```

The installer writes `install-manifest.json` and a generated `uninstall.exe` under `%LOCALAPPDATA%\Xpotato\devkit`, broadcasts PATH updates to newly opened Windows shells, warns if another `devkit.exe` is already present on PATH, and carries its own embedded `devkit.exe` payload for single-file redistribution.
