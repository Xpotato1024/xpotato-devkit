# xpotato-devkit

[![Rust Workflow](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml/badge.svg)](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

xpotato-devkit は、AI支援開発のための repo-agnostic な CLI ツールキットです。
現在の主実装は **Rust 製ネイティブ CLI** であり、高速性・配布性・安定した実行を重視しています。差分集計、ブロック抽出・置換、patch 診断、Git 補助のような deterministic な処理を CLI に寄せ、AI には判断と執筆だけをさせるのが狙いです。

> [!NOTE]
> Python版は後方互換のために同梱されていますが、現行の推奨実装および新機能追加の対象は Rust版 のみです。

## 何のツールか

`devkit` は、巨大な repo や長いファイルに対して、全文読込や全文再生成ではなく **局所探索 -> 局所編集 -> 検証** で進めるための CLI です。

- 差分の全体像は `devkit diff summarize`
- 探索の入口は `devkit search text` / `devkit search symbol`
- 対象箇所の絞り込みは `devkit tree` / `devkit block outline` / `devkit block context`
- 抽出と置換は `devkit block extract` / `devkit block replace`
- patch は `devkit patch diagnose` で診断してから `devkit patch apply`

詳しい価値説明は [Why devkit](docs/concepts/why-devkit.md) を参照してください。

## 何を防ぐか

`devkit` は次のような失敗を減らすためのツールです。

- AI が長いファイルを毎回全文読む
- 関数 1 個の修正なのにファイル全体を書き直す
- patch が外れているのに、そのまま適用しようとする
- Markdown の 1 セクションだけ直せばよいのに全文置換する
- Windows で `Get-Content` や `Get-ChildItem -Recurse` を乱発してノイズを増やす
- `rg` や `Select-String` に戻って、検索結果から次の操作が分断される
- `git diff` や `tree` の生出力に戻って、機械処理しづらい情報を何度も読み直す

## 誰に向くか

- **AI エージェント支援開発**: `--brief` や JSON を使って、構造化された出力を次の処理に渡したい場合
- **人間による局所編集**: 大規模 repo でも対象箇所だけを素早く見て直したい場合
- **Windows / PowerShell 代替**: `Get-Content` や `Get-ChildItem` の全文読込・再帰列挙を減らしたい場合

## 3 分導線

まずは次の 5 手順だけ押さえれば使い始められます。

```bash
# 1. 変更の全体像を把握する
devkit diff summarize --name-status --limit 20

# 2. 文字列やシンボルで探索する
devkit search text "patch diagnose" --glob "*.md" --context 2
devkit search symbol PatchDiagnostic --type rust

# 3. 対象ファイルの構造や文脈を局所化する
devkit block outline rust/crates/devkit-cli/src/main.rs
devkit block context rust/crates/devkit-cli/src/main.rs format_timing_output

# 4. patch を適用前に診断する
devkit patch diagnose --patch-file .tmp/change.diff

# 5. 問題なければ適用する
devkit patch apply --patch-file .tmp/change.diff
```

この流れの背景は [AI agent workflow](docs/design/ai_agent_workflow.md) にまとめています。

## 主なコマンド群

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

Global flags currently implemented in Rust:

- `--brief`
- `--time`
- `--time-json`

Human-readable terminal output may use ANSI color on compatible terminals.
`--brief` and JSON output remain plain text so agents and scripts can parse them safely.

## インストール方法

### 1. Cargo を使ったインストール

Rust 開発環境がある場合、リポジトリから直接ビルド・インストールするのが最も最新です。

```bash
cargo install --git https://github.com/Xpotato1024/xpotato-devkit --branch main devkit-cli
```

ローカルクローン済みの場合は `cargo install --path rust/crates/devkit-cli` を使います。

もし `devkit --help` に `encoding` や `search` が出ない、または `devkit block extract --help` に `--list-headings` が出ない場合は、PATH 上の先頭 `devkit.exe` が古い可能性があります。現在の checkout を使って更新し、その後に `where devkit` で解決順を確認してください。

```bash
cargo install --path rust/crates/devkit-cli --force
```

Windows では `where devkit` の先頭に出るパスが実際に実行されます。更新後も古い binary が先頭に残っている場合は、その PATH 順序または配布元の binary を揃える必要があります。

### 2. バイナリのダウンロード (GitHub Releases)

Rust 環境がない場合は、[GitHub Releases](https://github.com/Xpotato1024/xpotato-devkit/releases) から各 OS 向けの配布物を取得してください。

- Windows: `devkit-installer-<tag>-x86_64-pc-windows-msvc.exe` または zip asset
- Linux / macOS: `devkit-<tag>-<target>.tar.gz`

Windows の installer 契約と `winget` 準備状況は [Windows winget prep](docs/install/windows-winget-prep.md) を参照してください。

## よく使うコマンド例

```bash
# ヘルプを表示
devkit --help

# 差分の状態だけを短く確認
devkit diff summarize --name-status --limit 20

# テキストやシンボルを検索
devkit search text "safe-push" --glob "*.md" --context 2
devkit search symbol PatchDiagnostic --type rust

# ディレクトリ構造を浅く確認
devkit tree --max-depth 2 --limit 40

# UTF-8/BOM/改行混在などの異常を要約チェック
devkit encoding check README.md docs/**/*.md --brief

# repo-bundled skill pack を別 workspace に展開
devkit bootstrap sync-skills --repo-root . --target ../target-repo

# repo-bundled skills 向けの AGENTS.md テンプレートを書く
devkit bootstrap init-agents --path ../target-repo/AGENTS.md

# BOM 除去と改行コード統一
devkit encoding normalize README.md docs/**/*.md --newline crlf

# Markdown 編集は既存ファイルの改行形式を維持
# ファイル全体を明示的に揃えるときは encoding normalize を使う

# Python ファイルの構造だけを見る
devkit block outline main.py --imports

# コミット文の下書きを生成
devkit git commit-message --staged

# 安全な push。upstream が無い場合は remote を明示
devkit git safe-push --remote origin --yes
```

## Legacy / Planned Scope

The items below reflect the broader project direction and legacy Python-era capability set, not the exact command surface of the current Rust binary.

- **`devkit encoding check [files...]`**: テキストファイルの UTF-8 妥当性、BOM の有無、置換文字や混在する改行コードを検査し、AI のハルシネーションを防止
- **`devkit diff summarize`**: カレントワークスペース、ステージ済み変更、または特定の比較範囲（`--base/--head`）の差分を要約・集計。`--files-only`、`--name-status`、`--stat`、`--limit` により最初の変更把握を短くできる
- **`devkit search text / search symbol`**: `rg` や `Select-String` に戻らず、`.gitignore` を踏まえてテキスト検索や宣言シンボル検索を行い、そのまま `block` 系へ接続
- **`devkit block extract / replace`**: 長大なファイルから行数や見出し、関数定義等を基準に特定のブロックだけを抽出・置換
- **`devkit block outline / context`**:
  - `outline`: ファイル中の関数/クラスのシグネチャのみを行番号付きで抽出し構造を把握
  - `context`: 指定シンボルの前後数行のコンテキストを行番号付きで表示し、patch 作成の文脈として提供
- **`devkit tree`**: プロジェクトのディレクトリ構造をコンパクトに表示。`.gitignore` や `devkit.toml` を尊重し、`--max-depth`、`--files-only`、`--dirs-only`、`--glob`、`--json` を使ってノイズを削る
- **`devkit md` 系**: Markdown の特定セクションに対する追記・置換・新規作成
- **`devkit doc` 系**: 実装メモやベンチマーク記録のテンプレートを自動生成
- **`devkit git` 系**: コミットメッセージ・PR テンプレート生成、安全なプッシュラッパー (`safe-push`) など
- **`devkit patch apply / diagnose`**: パッチ適用と診断。`diagnose` は invalid patch、target missing、context mismatch、already applied/reversed を分類し、JSON と brief の両方で安定出力

> **`--brief` モード**: 主要コマンドには `--brief` フラグがあり、出力を `OK: ...` / `FAIL: ...` の 1 行に制限することで、AI エージェントのトークン消費をさらに節約できます。

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

Bundled repo skills should prefer `--brief` or JSON-capable output when another tool or agent will consume the result, because the default human-facing terminal output may be colorized.
Use `devkit bootstrap sync-skills` to copy the repo-bundled `SKILLs/` tree into another workspace, and `devkit bootstrap init-agents` to generate a starter `AGENTS.md` for that workspace.

## Release

Release is tag-triggered only:

- Push a `v*` tag to build and publish Windows, Linux, and macOS Apple Silicon assets.
- Pushes to `main` and pull requests do not create releases.
- Windows releases publish both `devkit-{tag}-x86_64-pc-windows-msvc.zip` and `devkit-installer-{tag}-x86_64-pc-windows-msvc.exe`.
- Release outputs also include `devkit-{tag}-sha256.txt` for checksum verification.

Release note:

- `v0.1.6` is the checked release target for the first `winget` submission. The published Windows installer asset, checksum file, draft manifests, and silent install/uninstall verification are now aligned.

See [tag-triggered release docs](docs/release/tag-triggered-release.md) for the maintainer flow.

## Windows installation

Windows users can follow [docs/install/windows-installation.md](docs/install/windows-installation.md) for the native user-local install flow.
Linux and macOS release archives ship the `devkit` binary only and do not include a native installer.

The recommended Windows asset is `devkit-installer-{tag}-x86_64-pc-windows-msvc.exe`. By default it installs to `%LOCALAPPDATA%\\Xpotato\\devkit` and adds that directory to the user PATH. Use `--unpack-only` to skip PATH changes:

```powershell
.\devkit-installer.exe --unpack-only
```

For unattended installation, use `--silent`. `winget` preparation notes and draft manifests live under [docs/install/windows-winget-prep.md](docs/install/windows-winget-prep.md) and `packaging/winget/`.
