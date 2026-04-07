use crate::diff::{build_diff_scope, summarize_diff, summarize_diff_scope, DiffScope, FileDiff};
use crate::run_git_command;

fn is_japanese(lang: &str) -> bool {
    lang.to_lowercase().starts_with("ja")
}

fn format_file_summary(files: &[FileDiff]) -> String {
    if files.is_empty() {
        return "# - none".to_string();
    }
    let mut lines = Vec::new();
    let num = std::cmp::min(files.len(), 10);
    for entry in &files[..num] {
        let detail = if entry.is_binary {
            "binary".to_string()
        } else {
            format!("+{}/-{}", entry.additions, entry.deletions)
        };
        lines.push(format!("# - {} ({})", entry.path, detail));
    }
    let remaining = files.len().saturating_sub(num);
    if remaining > 0 {
        lines.push(format!("# - ... and {} more file(s)", remaining));
    }
    lines.join("\n")
}

fn format_lines_with_limit(lines: &[&str], limit: usize) -> String {
    if lines.is_empty() {
        return "# - none".to_string();
    }
    let mut formatted = Vec::new();
    let num = std::cmp::min(lines.len(), limit);
    for line in &lines[..num] {
        formatted.push(format!("# - {}", line));
    }
    let remaining = lines.len().saturating_sub(num);
    if remaining > 0 {
        formatted.push(format!("# - ... and {} more line(s)", remaining));
    }
    formatted.join("\n")
}

fn scope_comment(scope: &crate::diff::ScopeDesc) -> String {
    if let Some(ref r) = scope.refspec {
        format!("{} ({})", scope.description, r)
    } else {
        scope.description.clone()
    }
}

fn resolve_default_pr_scope() -> Result<DiffScope, String> {
    for candidate in &["origin/HEAD", "refs/remotes/origin/HEAD"] {
        if let Ok(resolved) = run_git_command(&["symbolic-ref", "--short", candidate]) {
            let res = resolved.trim();
            if !res.is_empty() {
                return build_diff_scope(false, Some(res), Some("HEAD"), None);
            }
        }
    }
    for candidate in &["origin/main", "origin/master", "main", "master"] {
        if run_git_command(&["rev-parse", "--verify", candidate]).is_ok() {
            return build_diff_scope(false, Some(candidate), Some("HEAD"), None);
        }
    }
    Err("Failed to determine a default PR base. Use `--base/--head` or `--commits`.".to_string())
}

fn resolve_requested_scope(
    staged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
    default_mode: &str,
) -> Result<DiffScope, String> {
    if default_mode == "pr" && !staged && base.is_none() && head.is_none() && commits.is_none() {
        return resolve_default_pr_scope();
    }
    build_diff_scope(staged, base, head, commits)
}

fn diff_stat_for_scope(scope: &DiffScope) -> Vec<String> {
    let output = if scope.mode == "staged" {
        run_git_command(&["diff", "--stat", "--staged"])
    } else if scope.mode == "unstaged" {
        run_git_command(&["diff", "--stat"])
    } else {
        run_git_command(&["diff", "--stat", scope.refspec.as_deref().unwrap_or("")])
    };
    output.unwrap_or_default().lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

fn log_lines_for_scope(scope: &DiffScope) -> Vec<String> {
    if scope.mode == "staged" || scope.mode == "unstaged" {
        return Vec::new();
    }
    if let Some(ref r) = scope.refspec {
        let log_refspec = if r.contains("...") { r.replacen("...", "..", 1) } else { r.clone() };
        let output = run_git_command(&["log", "--oneline", &log_refspec]).unwrap_or_default();
        output.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    } else {
        Vec::new()
    }
}

pub fn generate_commit_template(
    staged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
    lang: &str,
) -> Result<String, String> {
    let summary = summarize_diff(staged, base, head, commits)?;
    if summary.files.is_empty() {
        return Err("No changes found to generate commit message for.".to_string());
    }

    let prompt = if is_japanese(lang) {
        "あなたは Git のコミットメッセージを下書きするアシスタントです。\n\
        出力はコミットメッセージ本文のみ。\n\
        1行目: `Type: Subject` 形式で変更を要約する。\n\
        2行目: 空行。\n\
        3行目以降: 何を、なぜ変えたかを簡潔に箇条書きで書く。\n\
        推測は避け、下の要約にある事実だけを使う。"
    } else {
        "You are drafting a Git commit message.\n\
        Return only the commit message body.\n\
        Line 1: summarize the change as `Type: Subject`.\n\
        Line 2: blank.\n\
        Line 3+: briefly explain what changed and why.\n\
        Use only the factual context below."
    };

    Ok(format!("\
<!--
{}
-->

# Source scope: {}
# Diff summary: {} additions, {} deletions across {} file(s).
# File summary:
{}
", prompt, scope_comment(&summary.scope), summary.total_additions, summary.total_deletions, summary.files.len(), format_file_summary(&summary.files)))
}

pub fn generate_pr_template(
    staged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
    lang: &str,
) -> Result<String, String> {
    let scope = resolve_requested_scope(staged, base, head, commits, "pr")
        .map_err(|e| format!("Failed to collect diff context for the PR body. {}", e))?;
    let summary = summarize_diff_scope(&scope)
        .map_err(|e| format!("Failed to collect diff context for the PR body. {}", e))?;
    
    let stat_lines = diff_stat_for_scope(&scope);
    let log_lines = log_lines_for_scope(&scope);
    
    let stat_str_vec: Vec<&str> = stat_lines.iter().map(|s| s.as_str()).collect();
    let log_str_vec: Vec<&str> = log_lines.iter().map(|s| s.as_str()).collect();
    
    let formatted_log = format_lines_with_limit(&log_str_vec, 8);
    let formatted_stat = format_lines_with_limit(&stat_str_vec, 8);
    let scope_label = scope_comment(&summary.scope);
    
    if is_japanese(lang) {
        Ok(format!("\
## 概要
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

# Source scope:
# - {}

# Commits:
{}

# Diff Stat:
{}
-->", scope_label, formatted_log, formatted_stat))
    } else {
        Ok(format!("\
## Overview
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

# Source scope:
# - {}

# Commits:
{}

# Diff Stat:
{}
-->", scope_label, formatted_log, formatted_stat))
    }
}

pub fn check_safe_branch() -> Result<(), String> {
    let current = run_git_command(&["branch", "--show-current"]).unwrap_or_default();
    if current.is_empty() {
        return Err("Cannot push from a detached HEAD. Please checkout a branch first.".to_string());
    }
    if current == "main" || current == "master" {
        return Err(format!("Direct push to {} branch is highly discouraged. Please use a feature branch.", current));
    }
    Ok(())
}

pub fn check_upstream() -> bool {
    if let Ok(current) = run_git_command(&["branch", "--show-current"]) {
        if current.is_empty() { return false; }
        return run_git_command(&["rev-parse", "--abbrev-ref", &format!("{}@{{upstream}}", current)]).is_ok();
    }
    false
}

pub fn get_upstream_remote() -> Result<String, String> {
    let current = run_git_command(&["branch", "--show-current"])?;
    if current.is_empty() {
        return Err("No current branch.".to_string());
    }
    let upstream = run_git_command(&["rev-parse", "--abbrev-ref", &format!("{}@{{upstream}}", current)])?;
    let remote = upstream.split('/').next().unwrap_or(&upstream);
    Ok(remote.to_string())
}

pub fn get_current_branch() -> Result<String, String> {
    run_git_command(&["branch", "--show-current"])
}
