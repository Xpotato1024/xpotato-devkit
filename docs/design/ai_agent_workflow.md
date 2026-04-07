# AI エージェント向けワークフロー

`devkit` を AI エージェントが使うときは、全文を読むのではなく `inspect -> edit -> verify` を固定の流れにする。

## 1. Inspect

- 差分の全体像は `devkit diff summarize` で確認する。
- 機械処理しやすくしたいときは `--json` を使う。
- 人間やエージェントが短く確認するだけなら `--brief` を使う。
- 長いファイルは `devkit block outline`、`devkit block context`、`devkit block extract` で局所化する。

## 2. Edit

- 編集はできるだけ小さいブロック単位で行う。
- 置換前に `devkit patch diagnose <patch-file>` で適用可否を確認する。
- `patch` の失敗は、ファイル全体の再読込ではなく、診断結果をもとに修正する。
- Markdown なら `devkit md replace-section` などの局所コマンドを優先する。

## 3. Verify

- 編集後は `devkit patch apply <patch-file>` で適用する。
- 変更内容の確認は `devkit diff summarize` で再実施する。
- Git 向けの文面が必要なら `devkit git commit-message` と `devkit git pr-body` を使う。
- push 前には `devkit git safe-push --remote <name> --yes` を使う。

## 4. 出力契約

- `--brief` は 1 行の成功/失敗判定に使う。
- `--json` は後続ツールに渡すための構造化出力に使う。
- 両方が必要な場合は、まず `--json` を使い、要約表示は別途作る。
- 不確かな推測より、コマンドの結果を優先する。

## 5. 推奨順

1. `devkit diff summarize`
2. `devkit block extract` または `devkit block context`
3. `devkit patch diagnose`
4. `devkit patch apply`
5. `devkit git commit-message` / `devkit git pr-body`
6. `devkit git safe-push`
