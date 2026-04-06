# xpotato-devkit Rust移行実施計画書

## 1. 目的
本書は、`xpotato-devkit` を **安全に**、かつ **将来的な拡張性を維持したまま** Rust へ移行するための実施手順を定めるものである。

現行の `xpotato-devkit` は、repo-agnostic な AI 支援開発用 CLI として、差分要約、block 操作、outline/context、tree、Markdown 操作、記録テンプレ生成、patch 診断などを提供している。 fileciteturn22file0  
また、テスト結果では高頻度コマンド群の大半において **Python 起動コストが支配的** であり、部分的なロジック最適化よりも **CLI 実行単位のネイティブ化** に意味があることが示された。

本計画書の目標は、次の 4 点である。

- 現行 Python 版を壊さずに Rust 版へ移行する
- master を常に利用可能な状態に保つ
- 比較検証可能な二重実装期間を設ける
- 将来的な機能追加や再設計に耐える構造にする

---

## 2. 基本方針

### 2.1 いきなり全面置換しない
Python 版 CLI を即時削除するのではなく、Rust 版を段階的に追加し、一定期間は **並行運用** する。

### 2.2 master は常に安定維持
master には、少なくとも次を満たすものだけを入れる。

- Python 版が従来通り動作する
- Rust 版が追加されても既存ワークフローを壊さない
- ドキュメントとコマンド体系が追従している
- 比較ベンチマークが再現可能である

### 2.3 高頻度コマンドから移行する
移行優先度は「技術的に簡単な順」ではなく「高頻度かつ startup 支配で効果が大きい順」とする。

### 2.4 互換性を明示的に管理する
Python 版と Rust 版の出力差分・仕様差分・未実装項目は、放置せずドキュメント化し、受け入れ条件を明確にする。

---

## 3. 移行後の到達イメージ

最終的な目標構成は次の通りとする。

```text
repo root
├─ python/                # 既存 Python 実装（移行期間中は維持）
│  └─ src/devkit/...
├─ rust/                  # Rust 実装
│  ├─ Cargo.toml
│  └─ crates/
│     ├─ devkit-cli/
│     ├─ devkit-core/
│     ├─ devkit-block/
│     ├─ devkit-md/
│     └─ devkit-patch/
├─ docs/
│  ├─ rust-migration-plan.md
│  ├─ rust-parity-matrix.md
│  ├─ benchmark/
│  └─ decisions/
└─ scripts/
   ├─ compare_python_rust.*
   └─ benchmark_devkit.*
```

### 3.1 Rust 側の分割方針
最初から巨大な 1 バイナリに実装を詰め込むより、次のように責務分割する。

- `devkit-cli`: clap などを用いた CLI 層
- `devkit-core`: 共通型、エラー、出力整形、brief/timing
- `devkit-block`: outline/context/extract/replace 系
- `devkit-md`: Markdown section 操作
- `devkit-patch`: patch diagnose/apply 系
- `devkit-tree`: tree 走査系
- `devkit-git`: git 連携系

この分割により、
- テストしやすい
- 後から library 化しやすい
- daemon 化や MCP 化に転用しやすい
- 将来一部だけ WASM / library / service 化しやすい

という利点がある。

---

## 4. Git運用方針

## 4.1 ブランチ戦略
安全性を優先し、以下のブランチを用いる。

### 永続ブランチ
- `master`
  - 常に安定
  - リリース可能状態を維持
- `rust-main`
  - Rust 移行の統合ブランチ
  - parity 未達でもよい
  - ただしビルドと基礎テストは必須

### 短命ブランチ
- `feature/rust-cli-skeleton`
- `feature/rust-tree`
- `feature/rust-block-outline`
- `feature/rust-block-context`
- `feature/rust-md`
- `feature/rust-patch-diagnose`
- `feature/rust-parity-tests`
- `feature/rust-bench-ci`

### 必要に応じて
- `release/rust-preview`
- `hotfix/...`

## 4.2 マージ方針
- 日常開発は **feature → rust-main**
- parity と安定条件を満たした単位で **rust-main → master**
- master への直接 push は禁止
- 既存の `devkit git safe-push` の思想とも整合させ、保護ブランチ設定を有効化するのが望ましい。現行 CLI も `main` / `master` への直接 push 防止を提供している。 fileciteturn22file0

## 4.3 推奨ブランチ保護設定
GitHub 側で少なくとも次を設定する。

- `master` への直接 push 禁止
- PR 必須
- status check 必須
- 可能なら squash merge または rebase merge に統一
- force push 禁止
- branch deletion 制限
- CODEOWNERS またはセルフレビュー手順の明文化

---

## 5. 移行フェーズ

## Phase 0: 移行準備
目的:
- Rust 導入の前提整備
- 比較可能な土台づくり

### 実施内容
1. `rust-main` ブランチ作成
2. `docs/rust-migration-plan.md` 作成
3. `docs/rust-parity-matrix.md` 作成
4. 現行 Python コマンド一覧を固定
5. ベンチマーク再現スクリプト整備
6. `--time` / `--time-json` 仕様を固定
7. 出力フォーマット契約を決める
   - 通常出力
   - `--brief`
   - エラー形式
   - JSON 出力

### 受け入れ条件
- Rust 用ワークスペースが作成済み
- Python / Rust を比較するための docs が存在
- コマンド互換性確認対象が明文化されている

---

## Phase 1: Rust CLI 骨格作成
目的:
- 実行可能な Rust バイナリを用意
- Python 版と共存させる

### 実施内容
1. `rust/devkit-cli` 作成
2. CLI パーサ導入（例: `clap`）
3. 共通エラー型・出力ラッパ作成
4. `--brief`
5. `--time`
6. `--time-json`
7. サブコマンドだけ空実装で定義
   - `tree`
   - `block`
   - `md`
   - `patch`
   - `doc`
   - `git`

### コマンド名方針
移行期間中は衝突回避のため、最初は例えば次のように分ける。

- Python 版: `devkit`
- Rust 試験版: `devkit-rs`

最終的に parity 達成後に Rust 側へ正式な `devkit` 名を譲る。

### 受け入れ条件
- `devkit-rs --help` が動く
- 共通オプションが動く
- CI で Rust ビルド成功
- Python 版に影響なし

---

## Phase 2: 高頻度コマンドの先行移植
目的:
- startup 支配の高頻度コマンドから効果を取りにいく

### 優先順位
1. `tree`
2. `block outline`
3. `block context`
4. `block extract`
5. `patch diagnose`

### 理由
これらは短く高頻度で叩かれやすく、かつ今回の計測では startup 支配だったため、ネイティブ化の効果が最も出やすい。

### 実施内容
- Rust 実装を追加
- Python 版との比較テストを作成
- 出力差分を parity matrix に記録
- ベンチ再測定を実施

### 受け入れ条件
- 機能 parity が許容範囲
- ベンチで明確な改善
- 既存利用フローで致命的差異がない

---

## Phase 3: Markdown / Patch / Block replace 移植
目的:
- 実用度を高める
- Gale 的な「実装 + 記録更新」フローを Rust 版で再現可能にする

### 対象
- `md append-section`
- `md replace-section`
- `md ensure-section`
- `md append-bullet`
- `patch apply`
- `block replace`

### 注意点
このフェーズから「読む系」だけでなく「書く系」が増えるため、安全性要件を強化する。

### 必須対応
- `--dry-run`
- unified diff preview
- 書き換え前バックアップまたは rollback 補助
- frontmatter 保持テスト
- 改行コード保持テスト
- 文字コード安全性テスト

### 受け入れ条件
- 破壊的変更系に対する安全テストが通る
- Python 版と同等以上の安全性がある
- rollback しやすい設計になっている

---

## Phase 4: Git / Doc 系移植
目的:
- CLI 全体の本格移行を完成に近づける

### 対象
- `diff summarize`
- `doc impl-note`
- `doc benchmark-note`
- `git commit-message`
- `git pr-body`
- `git safe-push`

### 注意点
git 系は Python より subprocess や外部状態依存が大きいので、速度だけでなく仕様整合性を重視する。

### 受け入れ条件
- 主要ワークフローが Rust 版だけで完結する
- docs が Rust 版前提へ更新可能
- CI で Windows/Linux の主要ケースが通る

---

## Phase 5: 切替・整理
目的:
- 正式に Rust 版を主実装へ切り替える

### 実施内容
1. `devkit-rs` を `devkit` に昇格
2. Python 版は `devkit-py` 等で互換維持するか検討
3. README を Rust 版中心に更新
4. Python 実装を legacy 扱いへ移行
5. 一定期間後に Python 版削除可否を判断

### 受け入れ条件
- Rust 版が主要コマンドで parity 達成
- ベンチ結果が十分改善
- 配布・インストール手順が安定
- ロールバック手順が確立済み

---

## 6. 互換性管理

## 6.1 parity matrix を作る
`docs/rust-parity-matrix.md` を用意し、各コマンドについて次を管理する。

- 実装状況
- 主要オプション対応状況
- 出力差異
- 未解決課題
- ベンチ改善結果
- master へ入れてよいかの判断

### 管理例
| command | python | rust | parity | notes |
|---|---|---|---|---|
| tree | yes | yes | 95% | ignore順序差あり |
| block outline | yes | yes | 100% | |
| block context | yes | yes | 100% | |
| patch diagnose | yes | yes | 90% | エラー文言調整中 |

## 6.2 出力互換レベル
完全一致だけを要求すると進まないため、互換性を 3 段階で扱う。

- **L1: 機能互換**
  - 結果の意味が同じ
- **L2: 実用互換**
  - AI / 人間のワークフローで置き換え可能
- **L3: 文字列互換**
  - 出力文言まで揃う

master 反映条件は原則 L2 以上とし、必要な箇所だけ L3 を目指す。

---

## 7. テスト方針

## 7.1 必須テスト
- 単体テスト
- snapshot テスト
- 破壊的変更コマンドの安全テスト
- 既存 benchmark シナリオ再実行
- Python/Rust 比較テスト
- Windows / Linux CI

## 7.2 比較テスト
Python 版と Rust 版で次を比較する。

- stdout
- stderr
- return code
- 出力件数
- 変更ファイルの中身
- dry-run diff
- `--brief` 結果

## 7.3 ベンチマーク
少なくとも次を継続測定する。

- `tree`
- `block outline`
- `block context`
- `block extract`
- `patch diagnose`
- `doc impl-note`

結果は `docs/benchmark/` に保存する。

---

## 8. CI/CD方針

## 8.1 master 向け必須
- Python テスト
- Rust テスト
- Windows ビルド
- Linux ビルド
- parity 比較テスト（対象限定でも可）
- benchmark smoke

## 8.2 rust-main 向け
- Rust fmt / clippy
- cargo test
- 主要コマンドの snapshot
- Python 比較テスト
- release artifact 試作

## 8.3 リリース準備
- preview release を作る
- Windows 向け単体 exe
- Linux 向けバイナリ
- 必要に応じて checksum
- インストール手順 docs 反映

---

## 9. ロールバック方針

Rust 移行では「切り戻せること」が重要である。

### 9.1 master 切替前
- Python 版を削除しない
- コマンド名を分ける
- Python 版を常に再実行可能にする

### 9.2 master 切替後
- しばらく Python 版を legacy として保持
- 緊急時は `devkit-py` へ退避可能にする
- 重要変更は feature flag または preview release を挟む

### 9.3 ロールバック条件
次のいずれかでロールバックを検討する。

- 高頻度コマンドで仕様差が大きい
- 書き換え系で安全性事故が出る
- Windows で安定しない
- 配布手順が複雑化し過ぎる
- 期待した速度改善が出ない

---

## 10. 当面の具体的な手順

### Step 1
`master` から `rust-main` を作成する。

### Step 2
以下のブランチを順に切る。
- `feature/rust-cli-skeleton`
- `feature/rust-tree`
- `feature/rust-block-outline`
- `feature/rust-block-context`
- `feature/rust-block-extract`
- `feature/rust-patch-diagnose`

### Step 3
各 feature は必ず `rust-main` に PR する。  
いきなり `master` へは入れない。

### Step 4
`docs/rust-parity-matrix.md` と `docs/benchmark/` を更新しながら比較を続ける。

### Step 5
高頻度コマンド群で十分な性能改善と実用互換が確認できたら、`rust-main` から `master` へ段階的にマージする。

### Step 6
Rust 版が主要ワークフローを満たした段階で、preview release を切る。

### Step 7
preview で問題がなければ、Rust 版を正式 `devkit` に昇格させる。

---

## 11. 最終結論
`xpotato-devkit` は、現状の計測結果から見て、**部分的なロジック移植ではなく CLI 実行単位のネイティブ化が必要**な段階にある。  
そのため Rust 移行は有力である。

ただし、安全性と拡張性を両立するには、

- master を安定維持する
- `rust-main` で統合する
- 高頻度コマンドから順に移行する
- parity matrix で互換性を管理する
- Python 版をすぐに捨てない
- benchmark と比較テストを継続する

という段階移行が不可欠である。

したがって、本移行は **「全面置換を前提としつつ、運用は段階的に進める」** 方針で実施するのが最も合理的である。
