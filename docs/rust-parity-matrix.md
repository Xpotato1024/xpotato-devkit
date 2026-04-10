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
| encoding check | yes | yes | L2 | Rust parity added on 2026-04-08 follow-up |
| encoding normalize | stub | yes | L2 | Rust implementation includes dry-run and newline style selection |
| bootstrap install-self | yes | yes | L2 | Rust implementation uses cargo install from current checkout |
| bootstrap sync-skills | no | yes | L2 | Rust implementation copies repo-bundled `SKILLs/` into a target workspace |
| bootstrap init-agents | no | yes | L2 | Rust implementation writes a starter `AGENTS.md` for repo-bundled skill usage |
| config init | no | yes | L2 | Rust CLI can generate an editable `devkit.toml` template |
| metrics show | yes | yes | L2 | Rust implementation reads local JSONL metrics when enabled |
| tree | yes | yes | L2 | Phase 2 完了 |
| block outline | yes | yes | L2 | Phase 2 完了 |
| block context | yes | yes | L2 | Phase 2 完了 |
| block extract | yes | yes | L2 | Phase 2 完了 |
| block replace | yes | yes | L2 | Phase 3 完了 |
| patch diagnose | yes | yes | L2 | Phase 2 完了 |
| patch apply | yes | yes | L2 | Phase 3 完了 |
| md append-section | yes | yes | L2 | Phase 3 完了 |
| md replace-section | yes | yes | L2 | Phase 3 完了 |
| md ensure-section | yes | yes | L2 | Phase 3 完了 |
| md append-bullet | yes | yes | L2 | Phase 3 完了 |
| diff summarize | yes | yes | L2 | Phase 4 完了 |
| doc impl-note | yes | yes | L2 | Phase 4 完了 |
| doc benchmark-note | yes | yes | L2 | Phase 4 完了 |
| git commit-message | yes | yes | L2 | Phase 4 完了 |
| git pr-body | yes | yes | L2 | Phase 4 完了 |
| git safe-push | yes | yes | L2 | Phase 4 完了 |
