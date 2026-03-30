# xpotato-devkit

`xpotato-devkit` は、AIによる開発ワークフローをサポートするための、リポジトリに依存しない汎用的なCLIツールキットです。
エンコーディングチェック、差分の抽出、指定したテキストブロックの抽出・置換、AIへのプロンプト生成等の決定論的（deterministic）なユーティリティを提供し、LLM使用時のトークン消費量を最小限に抑えつつ、安全性を高めることを目的として設計されています。

## 主な機能

- **`devkit encoding check [files...]`**: テキストファイルの UTF-8 妥当性、BOM付きの有無、置換文字（文字化け）や混在する改行コードを検査し、AIのハルシネーション（幻覚）を未然に防ぎます。
- **`devkit diff summarize`**: カレントワークスペースにおける差分の統計情報（追加行数、削除行数）をファイルごとに集計します。
- **`devkit block extract / replace`**: 長大なファイルから行数や見出し（Markdown）、関数定義等を基準に特定のブロックだけを動的に抽出または置換し、AIに渡すペイロードをコンパクトに保ちます。
- **`devkit git commit-message / pr-body`**: `git diff` や `git log` の統計情報を含むコンテキスト情報と指示文フォーマットを構築し、AIアシスタント（CursorやGale等）に高品質なコミットメッセージやPR本文のドラフトを生成させるためのテンプレートを自動的に作成します。
- **`devkit git safe-push`**: `main` や `master` ブランチへの直接Pushを阻止する等、フェイルセーフを設けた安全なプッシュラッパーです。

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
```

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
