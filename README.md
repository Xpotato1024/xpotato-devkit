# xpotato-devkit

[![Rust Workflow](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml/badge.svg)](https://github.com/Xpotato1024/xpotato-devkit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

xpotato-devkit は、AI支援開発のための repo-agnostic な CLI ツールキットです。
現在の主実装は **Rust 製ネイティブCLI** であり、高速性・配布性・安定した実行を重視しています。差分集計、ブロック抽出・置換、AI向けテンプレート生成補助などツールとして確定的（deterministic）なユーティリティを提供し、LLM使用時のトークン消費量を最小限に抑えつつ、安全性を高めることを目的としています。

> [!NOTE]
> Python版は後方互換のために同梱されていますが、現行の推奨実装および新機能追加の対象は Rust版 のみです。

## 何ができるか

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

## よく使うコマンド例

各機能は `devkit` コマンドを通じて実行します。

```bash
# ヘルプと使用可能なコマンド一覧を表示
devkit --help

# pythonファイルの構造（関数・クラスシグネチャ）のみツリー表示
devkit block outline main.py --imports

# コミット文の下書きを生成
devkit git commit-message --staged

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

プロジェクトのルートに `devkit.toml` を配置することで、挙動をカスタマイズできます。

```toml
[encoding]
# 検査対象から除外するディレクトリ・ファイル名
ignore = [".git", "node_modules", "dist", ".venv", "__pycache__"]

[git]
# AI連携機能におけるプロンプト言語
lang = "ja"  # 英語の場合は "en" を指定
```

## ライセンス

本ソフトウェアは [MIT License](LICENSE) のもとで公開されています。
