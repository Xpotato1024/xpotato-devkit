import subprocess
from typing import Any, Dict, List
from devkit.core.diff import summarize_diff

def run_git_command(args: List[str]) -> str:
    result = subprocess.run(["git"] + args, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        error_msg = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"Git command failed: {error_msg}")
    return result.stdout.strip()

def _is_japanese(lang: str) -> bool:
    return lang.lower().startswith("ja")

def _format_file_summary(files: List[Dict[str, Any]]) -> str:
    if not files:
        return "# - none"

    lines = []
    for entry in files[:10]:
        if entry["is_binary"]:
            detail = "binary"
        else:
            detail = f"+{entry['additions']}/-{entry['deletions']}"
        lines.append(f"# - {entry['path']} ({detail})")

    remaining = len(files) - len(lines)
    if remaining > 0:
        lines.append(f"# - ... and {remaining} more file(s)")
    return "\n".join(lines)

def _format_lines_with_limit(lines: List[str], limit: int = 10) -> str:
    if not lines:
        return "# - none"

    trimmed = lines[:limit]
    formatted = [f"# - {line}" for line in trimmed]
    remaining = len(lines) - len(trimmed)
    if remaining > 0:
        formatted.append(f"# - ... and {remaining} more line(s)")
    return "\n".join(formatted)

def generate_commit_template(staged: bool = True, lang: str = 'ja') -> str:
    summary = summarize_diff(staged=staged)
    if not summary["files"]:
        raise ValueError("No changes found to generate commit message for.")

    if _is_japanese(lang):
        prompt = (
            "あなたは Git のコミットメッセージを下書きするアシスタントです。\n"
            "出力はコミットメッセージ本文のみ。\n"
            "1行目: `Type: Subject` 形式で変更を要約する。\n"
            "2行目: 空行。\n"
            "3行目以降: 何を、なぜ変えたかを簡潔に箇条書きで書く。\n"
            "推測は避け、下の要約にある事実だけを使う。"
        )
    else:
        prompt = (
            "You are drafting a Git commit message.\n"
            "Return only the commit message body.\n"
            "Line 1: summarize the change as `Type: Subject`.\n"
            "Line 2: blank.\n"
            "Line 3+: briefly explain what changed and why.\n"
            "Use only the factual context below."
        )

    return f"""<!--
{prompt}
-->

# Diff summary: {summary['total_additions']} additions, {summary['total_deletions']} deletions across {len(summary['files'])} file(s).
# File summary:
{_format_file_summary(summary['files'])}
"""

def generate_pr_template(base: str = "main", lang: str = "ja") -> str:
    try:
        stat_output = run_git_command(["diff", "--stat", f"{base}...HEAD"])
        log_output = run_git_command(["log", "--oneline", f"{base}..HEAD"])
    except RuntimeError as e:
        raise ValueError(f"Failed to get diff against {base}. Is the branch name correct?") from e
    
    log_lines = log_output.split('\n')
    stat_lines = stat_output.split('\n')
    formatted_log = _format_lines_with_limit(log_lines, limit=8)
    formatted_stat = _format_lines_with_limit(stat_lines, limit=8)
    
    if _is_japanese(lang):
        tpl = f"""## 概要
* 変更の要点を1〜2文でまとめてください。

## 主な変更点
* 

## 確認事項
* [ ] 手元で動作確認をしたか

## 残課題・特記事項
* 

<!--
あなたは PR 本文を下書きするアシスタントです。
出力は Markdown 本文のみ。
上の各見出しを埋めること。
推測は避け、下の要約にある事実だけを使う。

# Commits:
{formatted_log}

# Diff Stat:
{formatted_stat}
-->"""
    else:
        tpl = f"""## Overview
* Summarize the change in 1-2 sentences.

## Key Changes
* 

## Checklist
* [ ] Tested locally

## Notes/Outstanding Issues
* 

<!--
You are drafting a pull request body.
Return Markdown only.
Fill in the sections above.
Use only the factual context below.

# Commits:
{formatted_log}

# Diff Stat:
{formatted_stat}
-->"""
    return tpl

def check_safe_branch() -> None:
    current = run_git_command(["branch", "--show-current"])
    if not current:
        raise RuntimeError("Cannot push from a detached HEAD. Please checkout a branch first.")
    if current in ("main", "master"):
        raise RuntimeError(f"Direct push to {current} branch is highly discouraged. Please use a feature branch.")

def check_upstream() -> bool:
    current = run_git_command(["branch", "--show-current"])
    if not current:
        return False
    try:
        run_git_command(["rev-parse", "--abbrev-ref", f"{current}@{{upstream}}"])
        return True
    except RuntimeError:
        return False
        
def get_current_branch() -> str:
    return run_git_command(["branch", "--show-current"])
