# xpotato-devkit Rust移行後の公開仕上げ計画

## 1. 目的

`xpotato-devkit` は、AI支援開発におけるトークン消費削減と、安全で確定的な編集補助を目的としたCLIツールキットである。  
現時点で実装の主軸は Rust ネイティブ CLI に移行済みであり、README でもインストール方法は `cargo install --path rust/crates/devkit-cli` を前提に説明されている。

したがって、今後の重点は **Rust化そのものの継続** ではなく、**外部公開時に「Rust製の実用CLI」として一貫して見える状態へ仕上げること** に置く。

---

## 2. 現状評価

## 2.1 良い点

- Rust workspace 化が完了している。
- CLI と機能別クレートの分離が進んでいる。
- README 上でも Rust を主実装として扱っている。
- release profile で `lto = true`, `codegen-units = 1` が設定されており、配布用CLIとしての意識がある。
- Python 版は `devkit-py` として後方互換に退避されており、移行方針自体は妥当である。

## 2.2 まだ残る課題

- リポジトリ root に `pyproject.toml`, `uv.lock`, `src/devkit` が残っており、初見では Python 主体に見える。
- GitHub 上の言語表示や表層構造が、Rust 移行済みの印象と一致しにくい。
- GitHub Releases など、非 Rust ユーザー向けの導線がまだ弱い。
- Rust 化の価値を示す公開ベンチマークが前面に出ていない。
- Python 版の保守範囲が、今後どこまでかを明文化したほうがよい。

---

## 3. 方針

今後の方針は次の4点に集約する。

1. **主役を Rust に固定する**
2. **導入導線を Cargo 前提だけに閉じず、バイナリ配布まで含めて整える**
3. **Python 版は legacy / compatibility layer として明確に位置づける**
4. **Rust 移行の価値を、性能・配布性・運用性で説明可能にする**

---

## 4. 優先度付き実施項目

## P0. 表層メッセージの統一

### 目的
「このプロジェクトは Rust 製CLIである」と初見で理解できる状態にする。

### 実施項目
- README 冒頭に以下を明記する
  - 現行主実装は Rust
  - Python 版は legacy / compatibility 用
  - 新機能追加は Rust 優先
- GitHub リポジトリの About を設定する
- GitHub Topics を追加する
  - `rust`
  - `cli`
  - `developer-tools`
  - `ai`
  - `llm`
  - `productivity`
- README 冒頭に「インストール」「主要ユースケース」「Python版の位置づけ」を簡潔に整理する

### 完了条件
- GitHub トップ画面だけ見ても、Rust CLI プロジェクトであることが伝わる
- README 冒頭 30 秒で利用者が導入方法を理解できる

---

## P1. 配布導線の整備

### 目的
Rust/Cargo を知らない利用者でも導入しやすくする。

### 実施項目
- GitHub Releases を開始する
- CI で主要プラットフォーム向けバイナリを生成する
  - Linux x86_64
  - Windows x86_64
  - macOS Apple Silicon
- README にインストール方法を3系統で整理する
  1. Cargo install
  2. GitHub Releases からバイナリ取得
  3. Python legacy 版（必要な場合のみ）
- 将来的に必要なら package manager 配布も検討する
  - Homebrew
  - Scoop
  - cargo-binstall 対応

### 完了条件
- Rust 未経験者にも導入経路がある
- README の install セクションで迷わない
- リリース版の実行ファイルがすぐ使える

---

## P2. Python legacy の整理

### 目的
二重保守のリスクを抑えつつ、移行の緩衝材としては機能させる。

### 実施項目
- Python 版の README / docs を legacy 扱いとして分離する
- ルート README では Python 版を主導線にしない
- Python 版について、次を明示する
  - 新機能追加対象外
  - 重大不具合修正のみ
  - 将来的な削除条件または維持方針
- `pyproject.toml` は必要なら残すが、位置づけを明文化する
- 可能であれば `legacy/python/` のようなディレクトリ分離を検討する

### 完了条件
- 利用者が Python 版を「現行版」と誤認しない
- メンテナが将来の保守負債を抱えにくい

---

## P3. ベンチマーク公開

### 目的
Rust 化の理由を感覚ではなく数値で示す。

### 実施項目
- Python 版と Rust 版の比較ベンチを公開する
- 少なくとも以下を計測する
  - 起動時間
  - `block extract`
  - `tree`
  - `diff summarize`
  - `patch diagnose`
- できれば以下も含める
  - 小規模 repo
  - 中規模 repo
  - 大規模 repo
- 指標は平均だけでなく p50 / p95 も出す
- README には短縮版、`docs/benchmarks/` には詳細版を置く

### 完了条件
- 「なぜ Rust なのか」に対して即答できる
- 配布性だけでなく性能面の説得力がある

---

## P4. CI / 品質保証の公開整備

### 目的
CLI ツールとしての信頼性を高める。

### 実施項目
- GitHub Actions で以下を整備する
  - Rust test
  - fmt
  - clippy
  - release build
  - cross-platform build
- 可能なら smoke test を追加する
  - `devkit --help`
  - 代表コマンドの簡易実行
- README に build status を表示する
- 変更方針として以下を docs に明文化する
  - 破壊的変更の扱い
  - CLI 出力互換性の考え方
  - brief モードの安定性方針

### 完了条件
- 「ローカルでは動く」ではなく「継続的に検証されるCLI」になっている
- リリース品質が一定以上に保たれる

---

## 5. 推奨ドキュメント構成

以下のように docs を整理すると、公開時の見通しが良い。

```text
docs/
  roadmap/
    rust-finish-plan.md
  benchmarks/
    python-vs-rust.md
  release/
    release-process.md
  legacy/
    python-legacy.md
  architecture/
    workspace-overview.md
```

### 各ドキュメントの役割
- `rust-finish-plan.md`
  - 本文書。公開仕上げの方針と優先順位。
- `python-vs-rust.md`
  - ベンチ結果の公開先。
- `release-process.md`
  - タグ付け、CI、成果物作成、チェックリスト。
- `python-legacy.md`
  - Python版の扱いと利用が必要なケース。
- `workspace-overview.md`
  - Rust workspace と各 crate の責務。

---

## 6. README 改訂の推奨構成

README は次の順に整理すると分かりやすい。

1. プロジェクト概要
2. 何ができるか
3. 最短インストール
4. よく使うコマンド例
5. Rust 版が現行であることの明示
6. Python legacy 版について
7. 設定ファイル
8. ライセンス

### 冒頭サンプル文案

```md
# xpotato-devkit

xpotato-devkit は、AI支援開発のための repo-agnostic な CLI ツールキットです。  
現在の主実装は Rust 製ネイティブ CLI であり、高速性・配布性・安定した実行を重視しています。

> Python 版は後方互換のために同梱されていますが、現行の推奨実装は Rust 版です。
```

---

## 7. リリース前チェックリスト

- [ ] README 冒頭が Rust 主体に統一されている
- [ ] GitHub About / Topics が設定済み
- [ ] GitHub Releases が作成されている
- [ ] Windows / Linux / macOS 向け成果物が生成される
- [ ] Python legacy の扱いが文書化されている
- [ ] ベンチマーク結果が公開されている
- [ ] CI で test / fmt / clippy / build が回る
- [ ] 主要コマンドの smoke test がある
- [ ] `devkit --help` ベースの初見体験が整理されている
- [ ] バージョニング方針が決まっている

---

## 8. 実行順の推奨

最小労力で見栄えと説得力を上げるなら、順番は以下がよい。

1. README / GitHub About / Topics 整理
2. GitHub Releases 開始
3. Python legacy 方針の文書化
4. CI で release build 自動化
5. Python vs Rust ベンチ公開
6. 必要に応じて package manager 対応

この順番なら、内部構造を大きく触らずに外部印象を大きく改善できる。

---

## 9. 結論

`xpotato-devkit` の Rust 移行は、技術的にはすでに十分成立している。  
今後の課題は「Rust で作れたか」ではなく、**Rust 製CLIとして外部にどう伝わるか** である。

したがって次のフェーズは、実装継続よりも以下を優先するべきである。

- 導線の一本化
- リリース配布
- legacy 整理
- 性能根拠の公開
- CI による品質保証

これらが揃えば、devkit は単なる個人用ツールではなく、AI時代の開発補助CLIとしてかなり説得力のある形になる。
