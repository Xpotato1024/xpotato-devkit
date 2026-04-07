"""Core logic for implementation/benchmark note template generation.

Generates Markdown note templates pre-filled with diff-derived context,
reducing the amount of boilerplate a developer (or AI) must write.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional


def _format_file_list(files: List[Dict[str, Any]], limit: int = 15) -> str:
    """Format a list of changed files as Markdown bullet items."""
    if not files:
        return "- (none)\n"
    lines: List[str] = []
    for entry in files[:limit]:
        if entry.get("is_binary"):
            detail = "binary"
        else:
            detail = f"+{entry['additions']}/-{entry['deletions']}"
        lines.append(f"- `{entry['path']}` ({detail})")
    remaining = len(files) - len(lines)
    if remaining > 0:
        lines.append(f"- ... and {remaining} more file(s)")
    return "\n".join(lines) + "\n"


def generate_impl_note(
    summary: Optional[Dict[str, Any]] = None,
    lang: str = "ja",
) -> str:
    """Generate an implementation note template.

    *summary* is the output of ``summarize_diff()`` (optional).
    """
    if lang.lower().startswith("ja"):
        return _impl_note_ja(summary)
    return _impl_note_en(summary)


def _impl_note_ja(summary: Optional[Dict[str, Any]]) -> str:
    scope_label = ""
    files_section = ""
    stats = ""
    if summary:
        scope = summary.get("scope", {})
        scope_label = scope.get("description", "")
        files_section = _format_file_list(summary.get("files", []))
        stats = f"+{summary.get('total_additions', 0)}/-{summary.get('total_deletions', 0)}"

    return f"""\
## 変更概要
<!-- 1-2文で変更の要点をまとめる -->


## 背景
<!-- なぜこの変更が必要か -->


## 実装内容
<!-- 主要な変更点を箇条書きで記述 -->


## 変更ファイル
{files_section}
{f"差分統計: {stats}" if stats else ""}
{f"スコープ: {scope_label}" if scope_label else ""}

## 検証
<!-- どのように動作確認したか -->
- [ ] テスト実行
- [ ] 手動確認

## 残課題
<!-- 未解決の問題や今後の改善点 -->

"""


def _impl_note_en(summary: Optional[Dict[str, Any]]) -> str:
    scope_label = ""
    files_section = ""
    stats = ""
    if summary:
        scope = summary.get("scope", {})
        scope_label = scope.get("description", "")
        files_section = _format_file_list(summary.get("files", []))
        stats = f"+{summary.get('total_additions', 0)}/-{summary.get('total_deletions', 0)}"

    return f"""\
## Summary
<!-- Summarize the change in 1-2 sentences -->


## Background
<!-- Why was this change needed? -->


## Changes
<!-- Key changes as bullet points -->


## Changed Files
{files_section}
{f"Diff stats: {stats}" if stats else ""}
{f"Scope: {scope_label}" if scope_label else ""}

## Verification
<!-- How was the change tested? -->
- [ ] Tests run
- [ ] Manual verification

## Outstanding Issues
<!-- Unresolved problems or future improvements -->

"""


def generate_benchmark_note(
    summary: Optional[Dict[str, Any]] = None,
    lang: str = "ja",
) -> str:
    """Generate a benchmark note template."""
    if lang.lower().startswith("ja"):
        return _benchmark_note_ja(summary)
    return _benchmark_note_en(summary)


def _benchmark_note_ja(summary: Optional[Dict[str, Any]]) -> str:
    files_section = ""
    if summary:
        files_section = _format_file_list(summary.get("files", []))

    return f"""\
## ベンチマーク概要
<!-- 計測の目的と対象を記述 -->


## 環境
<!-- OS, CPU, メモリ, ランタイムバージョンなど -->


## 手順
<!-- 再現手順を箇条書きで記述 -->


## 結果
| 項目 | Before | After | 変化率 |
|------|--------|-------|--------|
|      |        |       |        |

## 変更ファイル
{files_section}

## 考察
<!-- 結果の解釈と次のアクション -->

"""


def _benchmark_note_en(summary: Optional[Dict[str, Any]]) -> str:
    files_section = ""
    if summary:
        files_section = _format_file_list(summary.get("files", []))

    return f"""\
## Benchmark Summary
<!-- Purpose and target of the benchmark -->


## Environment
<!-- OS, CPU, memory, runtime version, etc. -->


## Procedure
<!-- Reproducible steps as bullet points -->


## Results
| Metric | Before | After | Change |
|--------|--------|-------|--------|
|        |        |       |        |

## Changed Files
{files_section}

## Analysis
<!-- Interpretation of results and next actions -->

"""
