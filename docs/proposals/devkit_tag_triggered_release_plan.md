# devkit Tag-Triggered Release Plan

## 目的

本ドキュメントは、`xpotato-devkit` のリリース運用を **「通常開発では一切 release せず、`v*` タグを明示的に切ったときのみ release する」** 方針で整備するための実装計画である。  
特に、コーディングエージェントの推論コストを下げるため、**曖昧な裁量を残さず、編集対象・実装順・判断基準・受け入れ条件を厳密に定義**する。

---

## 背景

現状のリポジトリでは、以下はすでに整っている。

- 既定ブランチは `main`
- CI は `main` 向けに Rust test / fmt / clippy / smoke test を実施
- README は Rust 主実装を明示
- README には GitHub Releases による配布導線がある
- README には「`v*` タグを push すると自動配布される」と書かれている

一方で、これまでの確認では、README のその説明を裏付ける release workflow の存在が確認できていない。  
そのため、**README の説明と実際の実装を一致させる**必要がある。

---

## 方針決定

### 採用方針

**タグ起点の明示的リリース運用**を採用する。

これは「完全手動」でも「常時自動公開」でもない。  
運用の実態は以下の通りとする。

- 通常の `main` push では release しない
- PR でも release しない
- `v*` 形式のタグを maintainers が明示的に push したときだけ release workflow が動く
- 生成された成果物は GitHub Releases に公開する
- 公開意思決定は「タグを切る人間」が持つ

### この方針を採る理由

1. 通常開発と公開を分離できる  
2. 「タグを切る」という明示的操作が最終判断になる  
3. 毎回の手動ビルド・手動添付より再現性が高い  
4. 完全自動公開より事故が少ない  
5. README の現在の説明と整合しやすい  

---

## 非目標

以下は今回のスコープ外とする。

- `main` push 時の自動 release
- nightly / canary / pre-release 自動配布
- Homebrew / Scoop / winget / apt 連携
- バイナリ署名
- SBOM 生成
- cargo-dist 導入
- changelog 自動生成の高度化
- semantic-release 相当の自動 versioning

必要最小限で、**README に書いてある内容を過不足なく実装する**ことを優先する。

---

## 実装対象

今回の実装対象は以下に限定する。

1. `.github/workflows/release.yml` の追加
2. `README.md` の release 説明の整合確認
3. 必要なら `docs/release/tag-triggered-release.md` の追加
4. リリースアセット命名規則の固定
5. リリース手順の明文化

---

## ディレクトリ・ファイル単位の変更方針

## 1. `.github/workflows/release.yml`

### 役割

`v*` タグ push をトリガーに、OS 別に Rust CLI をビルドし、成果物を GitHub Release に添付する。

### 必須要件

- `push.tags` で `v*` のみを受け付ける
- `branches` 起点では動かさない
- Linux / Windows / macOS の 3 OS を対象にする
- ビルド対象は `rust/crates/devkit-cli`
- 生成物名は後述の命名規則に従う
- Release が存在しなければ作成し、存在すれば失敗させるより整合的に扱う
- draft release ではなく通常 release でよい
- asset upload を行う
- workflow 名は `Release` とする

### 実装上の注意

- CI workflow (`ci.yml`) とは責務を分ける
- `main` push と `v*` tag push を混ぜない
- 「テストと release を 1 workflow に統合する」は不採用
- 将来の拡張性より、まずは単純さを優先する

---

## 2. `README.md`

### 役割

利用者に対して、「通常開発では release されず、`v*` タグ時のみ release される」ことを明確に伝える。

### 必須要件

- インストール方法の章は維持する
- GitHub Releases の説明は残す
- `v*` タグを明示的に作成・push したときだけ release が走ることを明記する
- `main` push では release されないことを読み取れる表現にする
- 曖昧な「自動で出ます」ではなく、タグ起点であることを書く

### 推奨表現

> GitHub 上で `v*`（例: `v0.1.0`）のタグを maintainer が明示的に作成・pushした場合にのみ、GitHub Actions が Windows / Linux / macOS 向けの実行バイナリをビルドし、Releases に公開します。通常の `main` push や pull request では release は行われません。

---

## 3. `docs/release/tag-triggered-release.md`

### 役割

maintainer 向けの運用手順書。README は利用者向けなので、内部運用の細部は docs に逃がす。

### 必須要件

以下を短く明記する。

- リリース前チェック
- タグ命名規則
- タグ作成コマンド例
- 失敗時の確認箇所
- release 後の確認項目
- ロールバック時の基本対応

### 非必須

- 詳細な GitHub UI スクリーンショット
- 複雑なフローチャート

---

## リリースアセット命名規則

アセット名は必ず固定する。  
コーディングエージェントが勝手に命名しないよう、以下を仕様とする。

## 形式

`devkit-{tag}-{target}{ext}`

## 例

- `devkit-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `devkit-v0.1.0-x86_64-pc-windows-msvc.zip`
- `devkit-v0.1.0-aarch64-apple-darwin.tar.gz` は **今回は対象外**
- `devkit-v0.1.0-x86_64-apple-darwin.tar.gz`

## 今回の対象 target

- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `x86_64-apple-darwin`

## 圧縮形式

- Windows: `.zip`
- Linux / macOS: `.tar.gz`

---

## 実装手順

以下の順序を固定する。  
コーディングエージェントはこの順番を崩さないこと。

## Step 1: 現行 README の release 記述を確認

目的は、後で workflow 実装と README の表現を一致させるためである。  
この段階では README をまだ編集しない。

### 完了条件

- README 内の release 説明位置を特定した
- 変更すべき文言が明確になった

---

## Step 2: `release.yml` を新規作成

`.github/workflows/release.yml` を追加する。  
まず workflow の最小骨格だけを作る。

### 必須内容

- `name: Release`
- `on.push.tags: ['v*']`
- 権限設定（contents write）
- matrix による 3 OS 定義
- checkout
- Rust toolchain setup
- release build
- アーカイブ生成
- release 作成または assets upload

### この段階でまだやらないこと

- README 編集
- docs 追加
- checksum 追加
- 複雑な conditional 分岐の導入

### 完了条件

- `release.yml` が構文として妥当
- 3 OS matrix が定義されている
- `v*` タグのみトリガーである

---

## Step 3: アーカイブ名と出力パスを固定

各 OS で生成するファイル名を統一する。  
ここを曖昧にすると、agent が毎回命名を変えやすい。

### 必須内容

- タグ名の取得方法を固定
- target triple を matrix で固定
- 出力先ディレクトリを一貫させる
- アーカイブ名を前節の仕様に一致させる

### 完了条件

- 3 OS すべてで命名規則が統一されている
- asset upload 側がそのファイル名を前提にしている

---

## Step 4: GitHub Release 作成・添付処理を実装

### 必須内容

- release 作成
- 同 release へのアセット添付
- release 名は tag 名でよい
- body は最小限でよい

### 許容

- body を簡素にする
- 初回は changelog 自動生成を入れない

### 非推奨

- 過剰なサードパーティ action の多用
- 実態が見えにくい composite action 化

### 完了条件

- `v*` タグ 1 回で release と 3 アセットが揃う構成になっている

---

## Step 5: `README.md` を実装に合わせて修正

workflow 実装後に README を修正する。順序を逆にしない。

### 必須内容

- タグ起点 release であることを明記
- `main` push / PR では release されないことを示す
- 利用者向け説明に留める

### 完了条件

- README の説明が workflow 実装と一致
- README を読んだだけで誤発火を想像しにくい

---

## Step 6: `docs/release/tag-triggered-release.md` を追加

maintainer 向け手順書を追加する。

### 必須内容

- 事前確認
- タグ作成例
- push 例
- 成果物確認
- 失敗時の確認項目
- 誤タグ時の対応方針

### 完了条件

- maintainers が README 以外の内部運用を docs で追える
- 手順が人依存になっていない

---

## Step 7: 静的確認

コード変更後、以下を確認する。

### 必須確認

- YAML 構文破綻がない
- CI workflow と release workflow の責務が分離されている
- `main` push と tag push のトリガーが混ざっていない
- README と docs の記述が一致している

### 完了条件

- diff を読んで方針のねじれがない

---

## 推奨実装詳細

以下は agent に明示してよい実装制約である。

## 使用方針

- Rust build は `working-directory: rust`
- CLI package は `devkit-cli`
- release build は `cargo build --package devkit-cli --release`
- binary 名は `devkit`（Windows は `devkit.exe`）

## GitHub Actions 設計方針

- 1 workflow 1責務
- release 用 workflow は tag trigger のみ
- `workflow_dispatch` は今回は入れない
- draft release は今回は使わない
- reusable workflow 化は今回はしない

## エラー処理方針

- 失敗時は workflow 自体を fail させる
- 黙ってスキップしない
- asset 名不一致も fail でよい

---

## 受け入れ条件

以下をすべて満たした場合のみ完了とみなす。

### 必須受け入れ条件

1. `.github/workflows/release.yml` が存在する  
2. trigger が `v*` タグ push のみである  
3. `main` push / PR では release しない  
4. Linux / Windows / macOS の 3 OS アセットを生成する  
5. アセット命名規則が文書と一致する  
6. README に tag-triggered release の説明がある  
7. README の説明が実装と矛盾しない  
8. maintainer 向け docs が追加されている  
9. CI workflow と release workflow が別ファイルで分離されている  
10. diff を読んだ第三者が「通常 push で勝手に公開される」と誤解しにくい  

### 望ましい受け入れ条件

11. release asset に実行ファイル以外の不要物が含まれない  
12. docs に誤タグ時の対処が書かれている  
13. README から docs/release へ導線がある  

---

## テスト計画

## 最低限の確認

- `main` への通常 push では release workflow が起動しないことを確認
- テスト用タグ（例: `v0.0.0-test`）で release workflow が起動することを確認
- 3 OS 分のアセットが揃うことを確認
- README の説明が事実と一致することを確認

## 実タグ確認時の注意

- 本番タグ命名規則を崩さない
- テストタグを残すか削除するか事前に決める
- 不要 release が残る場合の削除方針を docs に記す

---

## ロールバック方針

問題が起きた場合は以下の順で対処する。

1. 誤タグを削除する
2. 不要 release を削除または下書き化する
3. `release.yml` を修正する
4. README の説明が実装とズレたまま残らないようにする

---

## コーディングエージェントへの実行指示テンプレート

以下の方針で実装せよ。

1. 既存の `ci.yml` は変更しない  
2. `.github/workflows/release.yml` を新規追加せよ  
3. trigger は `push.tags: ['v*']` のみとせよ  
4. Linux / Windows / macOS の 3 OS matrix で `devkit-cli` release build を実行せよ  
5. 成果物を `devkit-{tag}-{target}{ext}` 形式でアーカイブ化せよ  
6. GitHub Release を作成し、各アセットを添付せよ  
7. その後に README の release 説明を tag-triggered release に合わせて修正せよ  
8. `docs/release/tag-triggered-release.md` を新規追加し、maintainer 向け手順を記述せよ  
9. scope 外の改善は入れるな  
10. 変更後、受け入れ条件を 1 つずつ自己点検せよ  

---

## 結論

今回の最適解は、**タグを切る人間が公開判断を持ちつつ、タグ push 後のビルド・配布だけを自動化する**ことにある。  
これは完全手動より再現性が高く、常時自動公開より安全であり、現在の README 方針とも整合する。

したがって、次に実装すべきなのは「release 自動化の全面導入」ではなく、**`v*` タグ限定の明示的 release workflow を最小構成で追加し、README と docs を一致させること**である。
