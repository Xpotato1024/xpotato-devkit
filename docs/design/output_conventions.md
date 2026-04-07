# xpotato-devkit 出力フォーマット・規約

本ドキュメントでは、`xpotato-devkit` の CLI 出力に対する基本的な設計ポリシーと規約を定義する。

## 1. `--brief` 出力フォーマット

AI エージェントが実行結果の手短な要約だけを必要とする場合のためのモード。

- **成功時**: `OK: <結果の1行サマリ>`
- **失敗時**: `FAIL: <原因の1行サマリ>`
- `len(lines) == 1` または最小行数であることを期待される。
- 色付け（ansi escape sequence）は出力されないこと（純粋なテキスト）。
- dry-run 指定可能なコマンドで dry-run 実行された場合は末尾に `(dry-run)` を付加する。

## 2. `--json` 出力との排他設定

- `--json` は構造化データを出力する目的であり、`--brief` とは目的が異なる。
- `--brief` と `--json` は原則として同時指定しない。
- `devkit diff summarize` と `devkit patch diagnose/apply` は、両方が指定された場合にエラーとする。

## 3. `patch` 系の出力契約

- `devkit patch diagnose` と `devkit patch apply` は、失敗時でも安定した `FAIL:` 系の brief を返せるようにする。
- `--brief` は 1 行で成功/失敗と要約だけを返す。
- `--json` は `command`、`success`、`summary`、`total_hunks`、`applied_hunks`、`failed_hunks`、`errors`、`affected_files` を含む。
- いずれのモードでも、末尾に説明用のフッターを追加しない。

## 4. `--time` / `--time-json` 出力フォーマット

実行時間の計測とボトルネック特定に用いる。

- 結果は **標準エラー出力 (stderr)** に出力される。
- フォーマット (human): `[time] {total}ms ({category}: {ms}ms, ...)`
- フォーマット (json): `{"total_ms": {total}, "{category}_ms": {ms}, ...}`
- 計測される主なカテゴリ:
  - `git_ms`: git サブプロセス呼び出しにかかった累積時間
  - `io_ms`: ファイル読み書きにかかった累積時間
  - `render_ms`: ターミナル描画関連にかかった時間（オプション）

## 5. Exit Code の意味

- `0`: 成功 (OK)
- `1`: 実行時エラー (FAIL) - パッチ適用失敗、エンコーディング検査での異常検出など
- `2`: コマンドライン引数エラー (typer のデフォルトの挙動)
