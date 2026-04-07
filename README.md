# xpotato-devkit

`xpotato-devkit` は、AIによる開発ワークフローをサポートするための、リポジトリに依存しない汎用的なCLIツールキットです。
差分集計、エンコーディング検査、ブロック抽出・置換、AI向けテンプレート生成補助などツールとして確定的（deterministic）なユーティリティを提供し、LLM使用時のトークン消費量を最小限に抑えつつ、安全性を高めることを目的として設計されています。

## 主な機能

- **`devkit encoding check [files...]`**: テキストファイルの UTF-8 妥当性、BOM付きの有無、置換文字や混在する改行コードを検査し、AIのハルシネーションを防ぎます。
- **`devkit encoding normalize`**: (予定/Stub) エンコーディングと改行コードを自動修正する将来の機能です。
- **`devkit diff summarize`**: カレントワークスペース、ステージ済み変更、または `--base/--head` で指定した比較範囲の差分を要約・集計します。`--files-only` でファイルパスのみの出力にも対応。
- **`devkit block extract / replace`**: 長大なファイルから行数や見出し（Markdown）、関数定義等を基準に特定のブロックだけを抽出・置換します。Python はインデントベース、Rust/Go/C/JS は波括弧ベースで関数終端を判定する言語対応の構造解析を備え、`--symbol` で struct/impl/enum も対象にできます。`--heading-exact` による完全一致指定や、同名見出しの disambiguation にも対応。`--list-headings` / `--list-functions` で探索を補助し、`--dry-run` では unified diff プレビューを表示します。
- **`devkit block outline`**: ファイル中の関数/クラスのシグネチャのみを行番号付きで抽出します。ボディを省略するため、ファイル全体を読まずに API 構造を把握でき、トークン消費を大幅に削減します。`--imports` で import 文、`--docstrings` で docstring 1 行目も含められます。
- **`devkit block context`**: 指定シンボルの前後 N 行を行番号付きで抽出します。`--margin` でコンテキスト量を制御でき、patch 作成の文脈として最適です。
- **`devkit tree`**: プロジェクトのディレクトリ構造をコンパクトに表示します。`.gitignore` と `devkit.toml` の ignore 設定を尊重し、`--max-depth`, `--ext`, `--dirs-only` によるフィルタリングに対応。AI がリポジトリ構造を把握する際のトークン消費を最小化します。
- **`devkit md append-section / replace-section / ensure-section / append-bullet`**: Markdown ドキュメントの特定セクションに対する追記・置換・新規作成・箇条書き追加を CLI で実行します。frontmatter を壊さず安全に操作し、`--dedupe` による重複除去にも対応します。
- **`devkit doc impl-note / benchmark-note`**: `diff summarize` の結果を自動注入した実装記録・ベンチマーク記録のテンプレートを生成します。日本語/英語の切替に対応。
- **`devkit git commit-message / pr-body`**: `git diff` や `git log` の要約をもとに、`--staged`、`--base/--head`、`--commits` で対象差分を絞った下書き用テンプレートを生成します。
- **`devkit git safe-push`**: `main` や `master` ブランチへの直接Pushを阻止し、`--yes` による非対話実行や `--remote` による upstream 設定に対応する安全なプッシュラッパーです。
- **`devkit patch apply / diagnose`**: unified diff パッチの適用と診断を行います。`--reject` による部分適用、`--verbose` による詳細出力、`diagnose` コマンドによる hunk 単位の適用可能性診断に対応し、AI による patch 再生成に使えるコンパクトなエラー要約を出力します。

> **`--brief` モード**: 上記の主要コマンドには共通で `--brief` フラグが用意されており、出力を `OK: ...` / `FAIL: ...` の 1 行に制限できます。AI エージェントが操作結果のみを確認する場合にトークンを大幅に節約します。

## インストール方法

本プロジェクトは現在 **Rust** によって完全に書き換えられ、高速なネイティブCLIとして動作します。
インストールには Cargo を使用します。

```bash
cargo install --path rust/crates/devkit-cli
```

各コマンドは `devkit` として利用可能になります。

```bash
# ヘルプと使用可能なコマンド一覧を表示
devkit --help

# Pythonファイルのエンコーディングと改行コードをチェック
devkit encoding check "*.py"

# ステージ済み変更だけを要約
devkit diff summarize --staged

# 比較範囲を明示してコミット文の下書きを生成
devkit git commit-message --base origin/main --head HEAD

# 長い Markdown の見出し一覧を確認
devkit block extract README.md --list-headings

# 安全な push。upstream が無い場合は remote を明示
devkit git safe-push --remote origin --yes
```

### レガシー (Python) 版の利用

以前の Python 版は後方互換性のため現在も同梱されています。Python版を利用する場合は [uv](https://github.com/astral-sh/uv) 経由でインストールし `devkit-py` プレフィックスを使用します:

```bash
uv sync
uv run devkit-py --help
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
