# AI エージェント改善実装レポート

更新日: 2026-04-08

## 実施内容

- `patch` 系コマンドに `--brief` 相当の 1 行出力を追加した。
- `patch diagnose` / `patch apply` に `--json` 出力を追加した。
- `--brief` と `--json` の同時指定を `diff` / `patch` で明示的に拒否するようにした。
- `--brief` 実行時に末尾の余計なフッターを出さないようにした。
- AI エージェント向けのワークフロー文書を追加した。
- `SKILLs/` に inspect/edit/verify 用と git draft 用の skill を追加した。

## 変更ファイル

- `rust/crates/devkit-patch/src/lib.rs`
- `rust/crates/devkit-cli/src/main.rs`
- `docs/design/output_conventions.md`
- `docs/design/ai_agent_workflow.md`
- `SKILLs/devkit-inspect-edit-verify/SKILL.md`
- `SKILLs/devkit-git-drafts/SKILL.md`

## 検証

- これから Rust テストとフォーマット確認を実施する。

## 備考

- `devkit` の役割は抽出・置換・patch・要約に限定し、検索やリポジトリ固有解釈は追加していない。
