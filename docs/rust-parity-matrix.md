# Rust移行 Parity Matrix

本ドキュメントは、Python 版 `devkit` を Rust 版へ移行する際の機能互換性（Parity）を管理するものです。

## 出力互換レベル
- **L1**: 機能互換 (結果の意味が同じ)
- **L2**: 実用互換 (AIや人間のワークフローで置き換え可能)
- **L3**: 文字列互換 (出力文言まで揃う)
※ master反映条件は原則 L2 以上です。

## コマンド互換性一覧

| command | python | rust | parity | notes |
|---|---|---|---|---|
| tree | yes | yes | L2 | Phase 2 完了 |
| block outline | yes | yes | L2 | Phase 2 完了 |
| block context | yes | yes | L2 | Phase 2 完了 |
| block extract | yes | yes | L2 | Phase 2 完了 |
| block replace | yes | no | 0% | Phase 3 予定 |
| patch diagnose | yes | yes | L2 | Phase 2 完了 |
| patch apply | yes | no | 0% | Phase 3 予定 |
| md append-section | yes | no | 0% | Phase 3 予定 |
| md replace-section | yes | no | 0% | Phase 3 予定 |
| md ensure-section | yes | no | 0% | Phase 3 予定 |
| md append-bullet | yes | no | 0% | Phase 3 予定 |
| diff summarize | yes | no | 0% | Phase 4 予定 |
| doc impl-note | yes | no | 0% | Phase 4 予定 |
| doc benchmark-note | yes | no | 0% | Phase 4 予定 |
| git commit-message | yes | no | 0% | Phase 4 予定 |
| git pr-body | yes | no | 0% | Phase 4 予定 |
| git safe-push | yes | no | 0% | Phase 4 予定 |
