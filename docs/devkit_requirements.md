# 汎用開発支援CLI 要求定義書（草案）

## 1. 概要

本プロジェクトは、AI 併用開発における固定費削減を目的とした、**汎用開発支援 CLI ツール群**を提供する。  
主な対象は以下である。

- Git 周りの定型処理
- 文字化け確認やテキスト健全性確認
- 長いファイルに対する局所抽出・局所置換・patch 適用
- diff 要約
- コミットメッセージ / PR 本文の下書き生成

本ツールは **Gale 由来の課題** を出発点とするが、**Gale 固有の構造や artifact 名には依存しない**。  
任意の Git リポジトリに後付けできる、**repo-agnostic な CLI** として設計する。

---

## 2. 背景

現状の AI 併用開発では、以下のようなコストが継続的に発生している。

1. 長いファイルの部分編集が不安定で、全文再生成に逃げやすい
2. 文字化け確認や diff 要約のような定型処理にも推論を使っている
3. コミットや PR 作成補助まで会話ベースで処理している

これらは一時的な問題ではなく、今後も継続して発生する固定費である。  
そのため、AI が担うべき「判断」「比較」「執筆」と、ツールが担うべき「確認」「整形」「局所適用」を分離する必要がある。

---

## 3. 目的

本プロジェクトの目的は次の通りである。

1. AI に渡す前処理・後処理を CLI 化し、トークン消費を削減する
2. 長いファイルに対する編集を、全文再生成ではなく局所編集中心へ移行する
3. Git 周りの定型文生成と安全操作を標準化する
4. 任意のリポジトリで再利用可能な形にする
5. Gale のような検索系ツールと競合せず、補完関係を築く

---

## 4. 非目的

本プロジェクトでは、以下は当面の対象外とする。

- IDE / エディタプラグインの開発
- GUI アプリ化
- Gale 固有の検索アルゴリズムや artifact 読み取り
- 大規模 CI/CD 再設計
- 汎用ドキュメント CMS 機能
- LLM オーケストレーション基盤の構築
- すべての文章生成を自動化する高機能ライティングツール化

---

## 5. 想定ユースケース

### 5.1 AI 併用コード編集
- 長いファイルから対象ブロックだけを抽出する
- AI にそのブロックだけ修正させる
- 生成結果を局所置換または patch 適用で戻す
- diff 要約だけを AI / 人間に提示する

### 5.2 文字化け確認
- 保存済みドキュメントやコミットメッセージ案の UTF-8 妥当性を確認する
- BOM、置換文字、制御文字、改行コード混在を検知する

### 5.3 Git 周りの定型処理
- staged diff からコミットメッセージ案を生成する
- 現在の差分から PR 本文案を生成する
- 安全な commit / push を補助する

### 5.4 汎用リポジトリ導入
- 特定のディレクトリ構成に依存せず導入する
- 設定ファイルでテンプレやルールを上書きできる

---

## 6. 基本方針

### 6.1 汎用性
- 特定プロジェクトのファイル構成を前提にしない
- 入力は通常のファイル、ディレクトリ、Git リポジトリ、diff とする
- 出力は標準出力、ファイル、JSON を基本とする

### 6.2 局所編集優先
- 全文再生成ではなく、局所抽出・局所置換・小さな patch を優先する
- 編集対象は関数、見出し、マーカー、行範囲などで指定可能にする

### 6.3 deterministic 優先
- 文字化け確認、diff 集計、Git 情報取得は推論ではなく deterministic に処理する
- AI は判断や執筆に集中させる

### 6.4 段階的拡張
- まずは CLI として最小機能を提供する
- 後から設定ファイルやテンプレ拡張を追加する
- 過剰抽象化を避ける

---

## 7. 実装方針

## 7.1 言語方針
本プロジェクトの実装言語は **Python** を第一候補とする。  
依存管理と実行環境管理には **uv** を用いる。

理由:
- 初期実装速度が高い
- CLI 試作が容易
- テキスト処理や Git 連携が書きやすい
- 仕様変更への追従が速い

ただし、以下の条件に該当する場合は将来的な Rust 移行を検討する。

- 長大ファイル処理が支配的になる
- 多数ファイルへの繰り返し適用が多い
- 単体バイナリ配布の要求が強まる
- 性能が実運用のボトルネックになる

## 7.2 パッケージ管理
Python 依存管理は `uv` を用いる。

想定コマンド例:
```bash
uv init
uv add typer rich pydantic
uv run devkit --help
```

## 7.3 CLI 形式
単発スクリプト群ではなく、**サブコマンド型 CLI** を基本とする。

想定例:
```bash
uv run devkit encoding check doc/**/*.md
uv run devkit diff summarize --staged
uv run devkit block extract src/main.rs --function search_command
uv run devkit block replace src/main.rs --marker "## Editing Policy" --with-file new.txt
uv run devkit patch apply --patch-file change.diff
uv run devkit git commit-message --staged --output .tmp/commit.txt
uv run devkit git pr-body --base main --output .tmp/pr_body.txt
```

---

## 8. 機能要件

## 8.1 encoding 系

### 8.1.1 encoding check
対象:
- テキストファイル全般
- Markdown
- ソースコード
- コミットメッセージ用一時ファイル
- PR 本文用一時ファイル

最低要件:
- UTF-8 として読めるか確認
- BOM 有無の報告
- 置換文字 `�` の検知
- 制御文字の検知
- CRLF/LF 混在の検知
- 複数ファイル一括処理
- 結果の text / JSON 出力

コマンド例:
```bash
uv run devkit encoding check doc/**/*.md
```

### 8.1.2 text normalize
最低要件:
- BOM 除去
- 改行コード統一
- 末尾改行の正規化
- 必要に応じて dry-run

---

## 8.2 diff 系

### 8.2.1 diff summarize
最低要件:
- changed files 一覧
- 追加 / 削除行数
- staged / unstaged の選択
- text / JSON 出力
- PR 本文やコミットメッセージ下書きの入力として利用可能

コマンド例:
```bash
uv run devkit diff summarize --staged
```

---

## 8.3 block 系

### 8.3.1 block extract
最低要件:
- 関数名指定
- 開始 / 終了マーカー指定
- 見出し単位指定
- 行範囲指定
- 抽出失敗時の明示的エラー
- 標準出力またはファイル出力

コマンド例:
```bash
uv run devkit block extract src/main.rs --function search_command
uv run devkit block extract AGENTS.md --heading "## Editing Policy"
```

### 8.3.2 block replace
最低要件:
- 対象ブロックの一意性確認
- 抽出方式と同じ指定方法で置換可能
- dry-run
- 置換前後差分の簡易表示
- 失敗時に元ファイルを壊さない

コマンド例:
```bash
uv run devkit block replace src/main.rs --function search_command --with-file .tmp/new_block.rs
```

### 8.3.3 patch apply
最低要件:
- unified diff の適用
- dry-run
- 適用失敗時の明確なエラー
- 適用対象の確認

コマンド例:
```bash
uv run devkit patch apply --patch-file .tmp/change.diff
```

---

## 8.4 git 系

### 8.4.1 commit message draft
最低要件:
- staged diff を元に日本語または英語の件名案を生成
- 必要なら本文も生成
- ファイル出力対応
- 出力テンプレ切替可能

コマンド例:
```bash
uv run devkit git commit-message --staged --lang ja --output .tmp/commit.txt
git commit -F .tmp/commit.txt
```

### 8.4.2 pr body draft
最低要件:
- 差分要約から PR 本文案を生成
- 「概要 / 主な変更 / 確認事項 / 残課題」などの定型化
- ファイル出力対応
- 出力テンプレ切替可能

コマンド例:
```bash
uv run devkit git pr-body --base main --lang ja --output .tmp/pr_body.txt
```

### 8.4.3 safe wrappers
最低要件:
- main / master 直操作警告
- 現在ブランチ確認
- upstream 未設定時の補助
- commit / push 実行前確認

---

## 9. 非機能要件

### 9.1 クロスプラットフォーム
- Windows
- Linux
- WSL
を主対象とする。  
macOS は将来対応を視野に入れるが、初期必須ではない。

### 9.2 導入容易性
- `uv sync` で環境構築できる
- README の初期手順が短い
- 特定の IDE を要求しない

### 9.3 保守性
- コマンドごとに責務を分離する
- I/O とロジックを分離する
- JSON 出力で他ツール連携しやすくする

### 9.4 安全性
- 破壊的変更は dry-run を持つ
- 置換や patch 適用失敗時に原本を壊さない
- Git 危険操作には明示的な確認を入れる

### 9.5 拡張性
- 将来的に `devkit.toml` 等で設定可能にする
- テンプレや ignore ルールを外出しできる構造にする

---

## 10. Gale との境界

### 10.1 本 CLI が担うこと
- 抽出
- 置換
- patch 適用
- diff 要約
- 文字化け確認
- Git 定型処理

### 10.2 Gale が担うこと
- 高速検索
- 長いファイルから対象候補を見つけること
- Gale 固有 artifact の読取や解析
- Gale 固有開発フローの知識

### 10.3 境界方針
Gale は「探す」、本 CLI は「抜く・置換する・適用する・整形する」。  
両者を競合させず、補完関係にする。

---

## 11. ディレクトリ構成案

初期案:
```text
devkit/
  README.md
  pyproject.toml
  uv.lock
  src/
    devkit/
      __init__.py
      cli.py
      commands/
        encoding.py
        diff.py
        block.py
        patch.py
        git.py
      core/
        encoding.py
        diff.py
        block.py
        patch.py
        git.py
  tests/
  docs/
    requirements.md
    architecture.md
```

---

## 12. 開発フェーズ

## Phase 1: 最小価値提供
- encoding check
- diff summarize
- block extract
- block replace
- patch apply

理由:
- トークン削減効果が高い
- Gale の課題に直結する
- 汎用化しやすい

## Phase 2: Git 補助
- commit message draft
- pr body draft
- safe wrappers

## Phase 3: 設定対応
- `devkit.toml`
- テンプレ切替
- ignore ルール
- 出力フォーマット拡張

---

## 13. 完了条件

初期版の完了条件は以下とする。

- `uv sync` と `uv run devkit --help` で CLI が起動する
- encoding check が複数ファイルに対して実行できる
- diff summarize が staged diff を要約できる
- block extract / replace が少なくとも 2 種類以上の指定方式で動く
- patch apply が unified diff を dry-run 付きで扱える
- Git 周りの下書き生成がファイル出力できる
- README に導入手順と代表コマンド例が記載されている

---

## 14. リスク

### 14.1 過剰抽象化
最初から万能ツールにしようとすると重くなる。  
初期版は Gale で本当に必要な最小機能に絞る。

### 14.2 Gale 依存の混入
Gale 固有の命名やディレクトリ前提を入れると汎用性が崩れる。  
入出力は一般的なファイル、diff、Git 情報に限定する。

### 14.3 Python 実装の性能限界
将来、長大ファイル処理や多数ファイル反復で性能課題が出る可能性がある。  
その場合のみ、block / patch 系の Rust 移行を検討する。

---

## 15. 最終方針

本プロジェクトは、**Gale 発の課題を汎用 CLI として切り出した開発支援ツール** として進める。  
初期実装は **Python + uv** で行い、以下を優先する。

- encoding check
- diff summarize
- block extract
- block replace
- patch apply
- commit message / PR body draft

これにより、AI 開発の固定費を下げ、全文再生成ではなく局所編集中心の開発フローへ移行する。
