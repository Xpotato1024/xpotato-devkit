# AI エージェント向けワークフロー

`devkit` を AI エージェントが使うときは、全文を読むのではなく `inspect -> edit -> verify` を固定の流れにする。

## 1. Inspect

- 差分の全体像は `devkit diff summarize` で確認する
- 文字列や宣言シンボルの探索は `devkit search text` / `devkit search symbol` で確認する
- 機械処理しやすくしたいときは `--json` を使う
- 人間やエージェントが短く確認するだけなら `--brief` を使う
- 長いファイルは `devkit tree`、`devkit block outline`、`devkit block context`、`devkit block extract` で局所化する

## 2. Edit

- 編集はできるだけ小さいブロック単位で行う
- 置換前に `devkit patch diagnose <patch-file>` で適用可否を確認する
- `patch` の失敗は、invalid patch / target missing / context mismatch / already applied or reversed の分類を見て修正する
- Markdown なら `devkit md replace-section` などの局所コマンドを優先する

## 3. Verify

- 編集後は `devkit patch apply <patch-file>` で適用する
- 変更内容の確認は `devkit diff summarize` で再実施する
- Git 向けの文面が必要なら `devkit git commit-message` と `devkit git pr-body` を使う
- push 前には `devkit git safe-push --remote <name> --yes` を使う

## 4. 出力契約

- `--brief` は 1 行の成功/失敗判定に使う
- `--json` は後続ツールに渡すための構造化出力に使う
- 両方が必要な場合は、まず `--json` を使い、要約表示は別途作る
- 不確かな推測より、コマンドの結果を優先する

## 5. Windows での代替導線

- `git diff` の概要確認の代わりに `devkit diff summarize`
- `Select-String` の代わりに `devkit search text`
- `Get-ChildItem -Recurse` の代わりに `devkit tree`
- `Get-Content` の全文読込の代わりに `devkit block context` / `devkit block extract`
- shell ワンライナーよりも `--brief` と JSON 出力を優先する

## 6. search 導線

探索系の公開 API は次の形で実装されている。

- `devkit search text <pattern>`
- `devkit search symbol <name>`

AI エージェントは、まず `search` を使って対象を絞り、必要なブロックだけを `block` 系で読む。
これにより `rg`、`Select-String`、`Get-Content` の雑な全文読込へ戻りにくくする。

最低限使うオプション:

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

理想の導線は次の通り。

1. `devkit diff summarize`
2. `devkit search text` または `devkit search symbol`
3. `devkit block context` または `devkit block extract`
4. `devkit patch diagnose`
5. `devkit patch apply`
