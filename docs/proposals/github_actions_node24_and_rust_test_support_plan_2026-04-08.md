# GitHub Actions Node 24 対応と Rust テスト補助共通化 計画

Date: 2026-04-08

## 目的

次の 2 点を、場当たり対応ではなく継続運用できる形で解消する。

1. GitHub Actions 上の Node 20 deprecation warning
2. Rust テストコードに散在している一時ディレクトリ生成、環境変数排他、パス比較補助の重複

この計画は実装用の前提整理、受け入れ条件、実施順序を日本語で固定するためのものです。

## 背景

現在の workflow では以下の action version を使用している。

- `.github/workflows/ci.yml`
  - `actions/checkout@v4`
  - `actions/cache@v4`
- `.github/workflows/release.yml`
  - `actions/checkout@v4`
  - `actions/cache@v4`
  - `actions/upload-artifact@v4`
  - `actions/download-artifact@v4`
  - `softprops/action-gh-release@v1`

GitHub Actions の実行ログでは、`actions/checkout@v4` と `actions/cache@v4` に対して Node 20 deprecation warning が出ている。公式 action 側では Node 24 対応版が既に公開されている。

Rust 側では CI flake 対応の過程で、各 crate に以下のようなテスト補助が増えた。

- timestamp 依存の一時ディレクトリ生成
- `DEVKIT_CONFIG` を使うテスト用の mutex
- macOS の `/var` と `/private/var` 差分を吸収するパス比較

現状は問題回避としては機能しているが、同じ実装が複数 crate に散り、今後の変更漏れや再発の原因になる。

## スコープ

この計画の実装対象は以下とする。

- GitHub Actions workflow の Node 24 対応
- Rust テスト補助の共通化
- CI 再発防止の検証手順整備

この計画では以下はスコープ外とする。

- リリースアセット形式の変更
- Homebrew / Scoop / winget 連携
- Rust 本体機能の追加
- 本番コード側の大規模リファクタ

## 現状の問題点

### 1. Node 20 warning が workflow ごとに散在している

- CI は `checkout@v4` と `cache@v4` を使っている
- Release は `checkout@v4`、`cache@v4`、artifact action の古い major を使っている
- warning の出方が job ごとに異なるため、1 本ずつ直しても別 workflow に残りやすい

### 2. テスト補助が crate ごとに重複している

- `TempDir` 実装が crate ごとに存在する
- `AtomicU64` を使った suffix 生成も crate ごとに存在する
- env lock の有無や適用範囲が crate ごとにぶれる
- path 比較が文字列比較に戻ると macOS で再発する

### 3. process-wide state を使うテストが設計上わかりにくい

- `DEVKIT_CONFIG` を書くテストだけでなく、読むテストも排他が必要
- ルールがコード上に明文化されていないと再発しやすい

## 実装方針

## 1. GitHub Actions Node 24 対応

### 基本方針

- warning が出ている action を、Node 24 対応済み major に更新する
- hosted runner 前提なので、公式 action の最小 runner version 条件は満たせる前提で進める
- 単に `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` を足して延命するのではなく、action version 自体を更新する

### 推奨更新

- `actions/checkout`
  - `@v4` から `@v5` へ更新
  - `@v5` は Node 24 runtime 対応
  - `@v6` は credential の扱いが変わるため、まずは差分の小さい `@v5` を第一候補とする
- `actions/cache`
  - `@v4` から `@v5` へ更新
  - `@v5` は Node 24 runtime 対応
- `actions/upload-artifact`
  - `@v4` から `@v6` を候補にする
  - Release workflow 側だけの変更になるため、CI と分けて検証する
- `actions/download-artifact`
  - `@v4` から Node 24 対応 major へ更新する
  - `upload-artifact` と同じ generation に揃える

### 追加監査対象

- `softprops/action-gh-release@v1`
  - 今回の warning 原因としては直接出ていないが、release workflow に含まれるため監査対象に残す
- `dtolnay/rust-toolchain@stable`
  - warning が出ていないため変更必須ではない

## 2. Rust テスト補助共通化

### 基本方針

workspace 内に dev 用の小さな helper crate を追加する。

候補:

- `rust/crates/devkit-test-support`

この crate は production code からは使わず、各 crate の `dev-dependencies` からのみ参照する。

### 提供する補助機能

- `new_temp_dir(prefix: &str) -> TestTempDir`
  - `pid + timestamp + atomic counter` で衝突を避ける
  - Drop で削除する
- `env_lock() -> &'static Mutex<()>`
  - process-wide env を使うテストの共通 lock
- `assert_same_file_location(left: &Path, right: &Path)`
  - canonicalize した親ディレクトリ + filename で比較する
- 必要なら `ScopedEnvVar`
  - set/remove を guard に閉じる

### 置き換え対象

- `rust/crates/devkit-core/src/lib.rs`
- `rust/crates/devkit-encoding/src/lib.rs`
- `rust/crates/devkit-metrics/src/lib.rs`
- `rust/crates/devkit-bootstrap/src/lib.rs`
- `rust/crates/devkit-block/src/lib.rs`
- `rust/crates/devkit-installer/src/main.rs`
- `rust/crates/devkit-installer/src/manifest.rs`

### 補助方針

- env var を読む可能性のあるテストは、読むだけでも lock 対象に含める
- path 比較で `to_string_lossy()` ベースの assertion を新規に増やさない
- temp dir helper は timestamp 単独生成に戻さない

## 受け入れ条件

### GitHub Actions

- `.github/workflows/ci.yml` で Node 20 deprecation warning が出ない
- `.github/workflows/release.yml` でも同種 warning が出ない
- CI と Release の両方で action version の更新後も既存機能が維持される

### Rust テスト補助

- 上記対象 crate から重複した temp dir helper を除去できる
- `DEVKIT_CONFIG` を扱うテストは共通 helper を使う
- path alias 差分を吸収する比較 helper が共通化される
- 既存の CI 再現テスト群が通る

### 検証

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- flake が出やすかった crate 群を複数回反復
  - `cargo test -p devkit-core -p devkit-encoding -p devkit-metrics -p devkit-bootstrap -p devkit-block -p devkit-installer`

## 実装手順

1. `.github/workflows/ci.yml` と `.github/workflows/release.yml` の action 使用箇所を棚卸しする
2. Node 24 対応済み major を workflow ごとに更新する
3. Release workflow の artifact action は upload/download を同系統 major に揃える
4. `cargo test --workspace` と tag-triggered release の dry verification を行う
5. `devkit-test-support` crate を追加する
6. temp dir helper を各 crate から移して共通化する
7. env lock と path comparison helper を共通化する
8. 重複 helper を各 crate から削除する
9. flake 再現用の反復テストを実行する
10. README または `docs/` に「Rust テストで process-wide state を扱うルール」を追記する

## リスク

- `actions/checkout@v6` まで一気に上げると、Node 24 以外の挙動差分も同時に入る
- artifact action の major 更新は、Release workflow の挙動差分確認が必要
- helper crate 導入時に、各 crate の `dev-dependencies` 更新漏れが起こりうる

## 推奨する実装順

最小リスクで進めるなら、以下の順を推奨する。

1. CI workflow の `checkout` と `cache` を先に更新
2. Release workflow の `checkout` / `cache` / artifact action を更新
3. test helper crate を追加
4. metrics → core → encoding → bootstrap → block → installer の順で置換

## 参考

- [actions/checkout marketplace](https://github.com/marketplace/actions/checkout)
- [actions/cache README](https://github.com/actions/cache)
- [actions/upload-artifact README](https://github.com/actions/upload-artifact)
