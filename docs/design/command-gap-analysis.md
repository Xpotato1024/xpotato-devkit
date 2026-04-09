# Command Gap Analysis

この文書は、`devkit` が既存コマンドを完全再実装するのではなく、実利用を阻害している不足だけを埋めるための整理です。

## 方針

- 採用するのは、利用頻度が高く、`devkit` の局所化・安全性・機械可読性を強めるもの
- 採用しないのは、完全互換のためだけの低頻度オプションや再現コストの高いもの

## `git diff`

- 既存コマンドの強み: 生の差分をそのまま見られる
- `devkit` の強み: `--brief`、JSON、`--files-only`、`--name-status`、`--stat`、`--limit` により最初の変更把握を短くできる
- 今回埋めるギャップ:
  - `--unstaged`
  - `--name-status`
  - `--stat`
  - `--limit`
  - JSON に status / totals / truncated を含める
- 今回見送るギャップ:
  - 完全な unified diff 表示
  - rename 詳細表示の高機能化

## `tree`

- 既存コマンドの強み: 手軽に構造を一覧できる
- `devkit` の強み: `.gitignore` と `devkit.toml` を踏まえ、局所探索向けの絞り込みができる
- 今回埋めるギャップ:
  - `--files-only`
  - `--hidden`
  - `--glob`
  - `--json`
  - `--limit`
- 今回見送るギャップ:
  - 既存 `tree` との表示完全互換
  - あらゆる ignore 仕様の再現

## `rg`

- 既存コマンドの強み: 高速な全文検索
- `devkit` の強み: block 系と組み合わせた局所読込導線を作れる
- 今回埋めたギャップ:
  - `devkit search text <pattern>`
  - `devkit search symbol <name>`
  - `--glob`
  - `--type`
  - `--ignore-case`
  - `--fixed-strings`
  - `--context`
  - `--files-with-matches`
  - `--count`
  - `--limit`
  - `--json`
  - `--brief`
- 今回見送るギャップ:
  - reference search のような semantic search
  - `rg` の完全互換

## `Get-Content`

- 既存コマンドの強み: どの Windows 環境でも使いやすい
- `devkit` の強み: 必要なブロックだけを読み、全文読込を避けられる
- 代替導線:
  - `devkit block context`
  - `devkit block extract`
  - `devkit search text`
- 今回見送るギャップ:
  - 検索結果から `block context` / `block extract` を自動で束ねるワンショット導線

## `Get-ChildItem`

- 既存コマンドの強み: Windows でそのまま使える
- `devkit` の強み: `tree` 側でノイズ制御と JSON 契約を持てる
- 代替導線:
  - `devkit tree --max-depth`
  - `devkit tree --glob`
  - `devkit tree --files-only`
- 今回見送るギャップ:
  - shell one-liner の完全な柔軟性

## 優先順位

1. `diff summarize` を変更把握の第一選択にする
2. `tree` を PowerShell 再帰列挙の代替として最低限使える形にする
3. `search` を `devkit` 内の探索入口として定着させる
4. `patch diagnose` を全文再読込に戻らない説明力にする

## 今回未実装の候補

- search から `block context` / `block extract` への直接接続
- declaration search を超える symbol reference search
- `rg` の完全互換オプション群
