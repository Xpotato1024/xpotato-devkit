# Fix Midterm Extension Future Work

この文書は、`fix_midterm_extension` ブランチで今回実装しなかった機能、将来に送る改善、PR 後のフォロー項目を整理するためのものです。

## 現在の前提

- `devkit diff summarize` の高優先拡張は実装済み
- `devkit tree` の高優先拡張は実装済み
- `devkit search text` / `devkit search symbol` は実装済み
- `patch diagnose` の failure classification と JSON 拡張は実装済み
- 主要 command surface の parse / help / representative runtime 監査は実施済み

このため、以下は「今回の PR を止める未完」ではなく、「次の改善候補」です。

## 次に着手しやすい項目

### 1. search から block への直接接続

現状でも `search -> block context/extract -> patch diagnose` の導線は成立しているが、結果の受け渡しはまだ手動寄りです。

候補:

- `search` 結果からそのまま `block context` に渡しやすい selector 出力
- `search` JSON に block selector 相当の field を追加
- `search symbol` の結果から declaration block を直接抽出する補助コマンド

### 2. reference search の別 API

`search symbol` は declaration search のまま維持する。
reference search は意味が違うため、同じ command に混ぜない方がよい。

候補:

- `devkit search refs <name>`
- `devkit search symbol --references`

推奨:

- declaration search と reference search は別 subcommand に分ける
- heuristic grep ではなく、language-aware 実装か、少なくとも false positive を強く抑える設計を取る

### 3. search の型・ignore 設定の拡張

現状の `--type` は簡易 alias と拡張子ベースです。

候補:

- `devkit.toml` から search 用 ignore/type alias を読む
- hidden file の opt-in support
- binary skip の理由を JSON に詳細化
- `search text` の path-only mode と summary mode の強化

## 中期改善候補

### 4. patch diagnose の精度向上

現状の classification は `git apply --check --verbose` 出力の best-effort 解析です。

残課題:

- hunk failure と git error の対応付け精度向上
- already applied / reversed 判定のさらなる確度向上
- context mismatch 時の現行近傍表示をより直接的にする
- `patch apply --json` 側との payload 整合を詰める

### 5. tree/search の共通化

今回のブランチでは speed と diff 最小化を優先して、`tree` と `search` の walker/filter 実装を大きく共通化していません。

将来候補:

- path walk/filter の shared crate 化
- `glob` / `type` / ignore handling の一元化
- JSON summary shape の整合

### 6. command audit の自動化強化

現状は unit test、integration smoke、help/runtime audit を入れているが、将来はさらに coverage を上げられます。

候補:

- top-level だけでなく leaf subcommand 全件の runtime smoke
- exit code contract の表形式 fixture 化
- PATH 上の stale binary 検出を CI とは別にローカル監査コマンド化

## Release / Distribution のフォロー

### 7. tag-triggered release の実証

release workflow の変更はローカル確認済みですが、実 tag publish での検証は未了です。

残タスク:

- standalone installer asset の実 release 確認
- checksum artifact の実 upload 確認
- `devkit -V` / installer metadata / release asset naming の整合確認

### 8. winget 実提出

`packaging/winget/` の draft manifest は作成済みです。

残タスク:

- 実 version に合わせた manifest 更新
- `winget-pkgs` 向け validation
- PR 提出と reviewer feedback 対応

## PR 前の判断

現時点で把握している残課題はすべて non-blocking です。

PR を止めるべき条件は次のいずれかが新たに見つかった場合だけです。

- 既存 command が regression している
- PATH 上の `devkit` が現行 binary を指していない
- release docs と実 command surface に齟齬がある

今回のローカル検証では、上記 3 点は解消済みです。
