import subprocess
from typing import Optional, List
from devkit.core.diff import summarize_diff

def run_git_command(args: List[str]) -> str:
    result = subprocess.run(["git"] + args, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        error_msg = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"Git command failed: {error_msg}")
    return result.stdout.strip()

def generate_commit_template(staged: bool = True, lang: str = 'ja') -> str:
    summary = summarize_diff(staged=staged)
    if not summary["files"]:
        raise ValueError("No changes found to generate commit message for.")
        
    diff_args = ["diff"]
    if staged:
        diff_args.append("--staged")
    full_diff = run_git_command(diff_args)
    
    if lang == 'ja':
        prompt = (
            "以下のGitの差分を確認し、コミットメッセージを作成してください。\n"
            "1行目に50文字以内で変更内容（Type: Subject の形式）を要約し、\n"
            "3行目以降に「なぜ変更したか」「何を変更したか」を箇条書き等で簡潔に記載してください。"
        )
    else:
        prompt = (
            "Please create a commit message for the following Git diff.\n"
            "Summarize the change in the first line (Type: Subject, max 50 chars),\n"
            "and explain 'why' and 'what' was changed in the body starting from the third line."
        )
        
    diff_lines = full_diff.split('\n')
    formatted_diff = '\n'.join(f"# {line}" for line in diff_lines)
    
    return f"""<!--
{prompt}
-->

# Write your commit message above. The following is context for the AI and will be ignored by Git.
# Diff summary:
# {summary['total_additions']} additions, {summary['total_deletions']} deletions across {len(summary['files'])} files.
#
# Full diff:
# ```diff
{formatted_diff}
# ```
"""

def generate_pr_template(base: str = "main", lang: str = "ja") -> str:
    try:
        stat_output = run_git_command(["diff", "--stat", f"{base}...HEAD"])
        log_output = run_git_command(["log", "--oneline", f"{base}..HEAD"])
    except RuntimeError as e:
        raise ValueError(f"Failed to get diff against {base}. Is the branch name correct?") from e
    
    log_lines = log_output.split('\n')
    stat_lines = stat_output.split('\n')
    formatted_log = '\n'.join(f"<!-- # {line} -->" for line in log_lines)
    formatted_stat = '\n'.join(f"<!-- # {line} -->" for line in stat_lines)
    
    if lang == 'ja':
        tpl = f"""## 概要
* ここにプルリクエストの概要を記載してください

## 主な変更点
* 

## 確認事項
* [ ] 手元で動作確認をしたか

## 残課題・特記事項
* 

<!--
以下の情報を参考に、上記の項目を適切な内容で埋めてください。

# Commits:
{formatted_log}

# Diff Stat:
{formatted_stat}
-->"""
    else:
         tpl = f"""## Overview
* Briefly describe the purpose of this PR.

## Key Changes
* 

## Checklist
* [ ] Tested locally

## Notes/Outstanding Issues
* 

<!--
Please use the following information to fill out the sections above.

# Commits:
{formatted_log}

# Diff Stat:
{formatted_stat}
-->"""
    return tpl

def check_safe_branch() -> None:
    current = run_git_command(["branch", "--show-current"])
    if current in ("main", "master"):
        raise RuntimeError(f"Direct push to {current} branch is highly discouraged. Please use a feature branch.")

def check_upstream() -> bool:
    current = run_git_command(["branch", "--show-current"])
    try:
        run_git_command(["rev-parse", "--abbrev-ref", f"{current}@{{upstream}}"])
        return True
    except RuntimeError:
        return False
        
def get_current_branch() -> str:
    return run_git_command(["branch", "--show-current"])
