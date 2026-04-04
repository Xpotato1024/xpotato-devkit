# xpotato-devkit

`xpotato-devkit` は、AIによる開発ワークフローをサポートするための、リポジトリに依存しない汎用的なCLIツールキットです。
差分集計、エンコーディング検査、ブロック抽出・置換、AI向けテンプレート生成補助などツールとして確定的（deterministic）なユーティリティを提供し、LLM使用時のトークン消費量を最小限に抑えつつ、安全性を高めることを目的として設計されています。

## 主な機能

- **`devkit encoding check [files...]`**: テキストファイルの UTF-8 妥当性、BOM付きの有無、置換文字や混在する改行コードを検査し、AIのハルシネーションを防ぎます。
- **`devkit encoding normalize`**: (予定/Stub) エンコーディングと改行コードを自動修正する将来の機能です。
- **`devkit diff summarize`**: カレントワークスペース、ステージ済み変更、または `--base/--head` で指定した比較範囲の差分を要約・集計します。
- **`devkit block extract / replace`**: 長大なファイルから行数や見出し（Markdown）、関数定義（簡易ヒューリスティック検索 / best-effort）等を基準に特定のブロックだけを抽出・置換し、`--list-headings` / `--list-functions` で探索も補助します。
- **`devkit git commit-message / pr-body`**: `git diff` や `git log` の要約をもとに、`--staged`、`--base/--head`、`--commits` で対象差分を絞った下書き用テンプレートを生成します。
- **`devkit git safe-push`**: `main` や `master` ブランチへの直接Pushを阻止し、`--yes` による非対話実行や `--remote` による upstream 設定に対応する安全なプッシュラッパーです。

## インストール方法

本プロジェクトのパッケージおよび環境管理には [uv](https://github.com/astral-sh/uv) を使用しています。

```bash
uv sync
```

各種コマンドを実行するには `uv run` を使用してください:

```bash
# ヘルプと使用可能なコマンド一覧を表示
uv run devkit --help

# Pythonファイルのエンコーディングと改行コードをチェック
uv run devkit encoding check "*.py"

# ステージ済み変更だけを要約
uv run devkit diff summarize --staged

# 比較範囲を明示してコミット文の下書きを生成
uv run devkit git commit-message --base origin/main --head HEAD

# 長い Markdown の見出し一覧を確認
uv run devkit block extract README.md --list-headings

# 安全な push。upstream が無い場合は remote を明示
uv run devkit git safe-push --remote origin --yes
```

checkout 中の `devkit` をユーザーツールとして導入したい場合は、次のコマンドを実行してください:

```bash
uv run devkit bootstrap install-self
```

既存の `scripts/bootstrap_devkit.py` は互換ラッパーとして残してあり、同じ処理を呼び出します。

## 設定ファイル (`devkit.toml`)

プロジェクトのルートディレクトリに `devkit.toml` を配置することで、ツールキットの挙動をプロジェクトごとにカスタマイズすることができます。

```toml
[encoding]
# encoding check コマンドの検査対象から除外するディレクトリ・ファイル名
ignore = [".git", "node_modules", "dist", ".venv", "__pycache__"]

[git]
# commit-message や pr-body におけるAIへの指示文のデフォルト言語
lang = "ja"  # 英語の場合は "en" を指定
```

## ライセンス

本ソフトウェアは [MIT License](LICENSE) のもとで公開されています。
