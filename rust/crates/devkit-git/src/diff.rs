use crate::run_git_command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeDesc {
    pub mode: String,
    pub description: String,
    pub refspec: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiffScope {
    pub mode: String,
    pub description: String,
    pub selector_args: Vec<String>,
    pub refspec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileDiff {
    pub path: String,
    pub status: String,
    pub additions: usize,
    pub deletions: usize,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffSummary {
    pub scope: ScopeDesc,
    pub files: Vec<FileDiff>,
    pub total_files: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub binary_files: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedNumstat {
    path: String,
    additions: usize,
    deletions: usize,
    is_binary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedNameStatus {
    path: String,
    status: String,
}

pub fn build_diff_scope(
    staged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
) -> Result<DiffScope, String> {
    build_diff_scope_with_unstaged(staged, false, base, head, commits)
}

pub fn build_diff_scope_with_unstaged(
    staged: bool,
    unstaged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
) -> Result<DiffScope, String> {
    if commits.is_some() && (staged || unstaged || base.is_some() || head.is_some()) {
        return Err(
            "`--commits` cannot be combined with `--staged`, `--unstaged`, or `--base/--head`."
                .to_string(),
        );
    }
    if staged && unstaged {
        return Err("`--staged` and `--unstaged` cannot be combined.".to_string());
    }
    if (staged || unstaged) && (base.is_some() || head.is_some()) {
        return Err(
            "`--staged` or `--unstaged` cannot be combined with `--base/--head`.".to_string(),
        );
    }
    if (base.is_some() && head.is_none()) || (head.is_some() && base.is_none()) {
        return Err("`--base` and `--head` must be provided together.".to_string());
    }

    if let Some(commits) = commits {
        return Ok(DiffScope {
            mode: "commits".to_string(),
            description: format!("commit range {}", commits),
            selector_args: vec![commits.to_string()],
            refspec: Some(commits.to_string()),
        });
    }
    if let (Some(b), Some(h)) = (base, head) {
        let refspec = format!("{}...{}", b, h);
        return Ok(DiffScope {
            mode: "range".to_string(),
            description: format!("range {}...{}", b, h),
            selector_args: vec![refspec.clone()],
            refspec: Some(refspec),
        });
    }
    if staged {
        return Ok(DiffScope {
            mode: "staged".to_string(),
            description: "staged changes".to_string(),
            selector_args: vec!["--staged".to_string()],
            refspec: None,
        });
    }
    if unstaged {
        return Ok(DiffScope {
            mode: "unstaged".to_string(),
            description: "unstaged changes".to_string(),
            selector_args: Vec::new(),
            refspec: None,
        });
    }
    Ok(DiffScope {
        mode: "unstaged".to_string(),
        description: "unstaged changes".to_string(),
        selector_args: Vec::new(),
        refspec: None,
    })
}

pub fn summarize_diff(
    staged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
) -> Result<DiffSummary, String> {
    summarize_diff_with_options(staged, false, base, head, commits, None)
}

pub fn summarize_diff_with_options(
    staged: bool,
    unstaged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
    limit: Option<usize>,
) -> Result<DiffSummary, String> {
    let scope = build_diff_scope_with_unstaged(staged, unstaged, base, head, commits)?;
    summarize_diff_scope(&scope, limit)
}

pub fn summarize_diff_scope(
    scope: &DiffScope,
    limit: Option<usize>,
) -> Result<DiffSummary, String> {
    let numstat_args = build_diff_command_args(scope, &["--numstat"]);
    let numstat_refs: Vec<&str> = numstat_args.iter().map(|arg| arg.as_str()).collect();
    let numstat = run_git_command(&numstat_refs)?;
    let name_status_args = build_diff_command_args(scope, &["--name-status"]);
    let name_status_refs: Vec<&str> = name_status_args.iter().map(|arg| arg.as_str()).collect();
    let name_status = run_git_command(&name_status_refs)?;

    let merged = merge_file_diffs(&parse_numstat(&numstat), &parse_name_status(&name_status));
    let total_files = merged.len();
    let total_additions = merged.iter().map(|file| file.additions).sum();
    let total_deletions = merged.iter().map(|file| file.deletions).sum();
    let binary_files = merged.iter().filter(|file| file.is_binary).count();
    let (files, truncated) = apply_limit(&merged, limit);

    Ok(DiffSummary {
        scope: ScopeDesc {
            mode: scope.mode.clone(),
            description: scope.description.clone(),
            refspec: scope.refspec.clone(),
        },
        files,
        total_files,
        total_additions,
        total_deletions,
        binary_files,
        truncated,
    })
}

fn build_diff_command_args(scope: &DiffScope, format_args: &[&str]) -> Vec<String> {
    let mut args = Vec::with_capacity(1 + format_args.len() + scope.selector_args.len());
    args.push("diff".to_string());
    args.extend(format_args.iter().map(|arg| (*arg).to_string()));
    args.extend(scope.selector_args.clone());
    args
}

fn parse_numstat(output: &str) -> Vec<ParsedNumstat> {
    let mut files_changed = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split('\t');
        let adds = parts.next().unwrap_or("").trim();
        let dels = parts.next().unwrap_or("").trim();
        let fname = parts.next().unwrap_or("").trim();

        if adds.is_empty() || dels.is_empty() || fname.is_empty() {
            continue;
        }

        let additions = if adds != "-" {
            adds.parse().unwrap_or(0)
        } else {
            0
        };
        let deletions = if dels != "-" {
            dels.parse().unwrap_or(0)
        } else {
            0
        };

        files_changed.push(ParsedNumstat {
            path: fname.to_string(),
            additions,
            deletions,
            is_binary: adds == "-",
        });
    }

    files_changed
}

fn parse_name_status(output: &str) -> Vec<ParsedNameStatus> {
    let mut parsed = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let raw_status = parts[0].trim();
        let status = raw_status
            .chars()
            .next()
            .map(|ch| ch.to_string())
            .unwrap_or_else(|| "?".to_string());
        let path = parts.last().copied().unwrap_or("").trim();
        if path.is_empty() {
            continue;
        }

        parsed.push(ParsedNameStatus {
            path: path.to_string(),
            status,
        });
    }

    parsed
}

fn merge_file_diffs(numstat: &[ParsedNumstat], statuses: &[ParsedNameStatus]) -> Vec<FileDiff> {
    numstat
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let status = statuses
                .get(index)
                .map(|value| value.status.clone())
                .unwrap_or_else(|| "M".to_string());
            let path = statuses
                .get(index)
                .map(|value| value.path.clone())
                .unwrap_or_else(|| file.path.clone());

            FileDiff {
                path,
                status,
                additions: file.additions,
                deletions: file.deletions,
                is_binary: file.is_binary,
            }
        })
        .collect()
}

fn apply_limit(files: &[FileDiff], limit: Option<usize>) -> (Vec<FileDiff>, bool) {
    match limit {
        Some(limit) if limit < files.len() => (files.iter().take(limit).cloned().collect(), true),
        _ => (files.to_vec(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_staged_and_unstaged_together() {
        let err = build_diff_scope_with_unstaged(true, true, None, None, None).unwrap_err();
        assert!(err.contains("cannot be combined"));
    }

    #[test]
    fn rejects_unstaged_with_range() {
        let err = build_diff_scope_with_unstaged(false, true, Some("main"), Some("HEAD"), None)
            .unwrap_err();
        assert!(err.contains("`--staged` or `--unstaged`"));
    }

    #[test]
    fn parses_name_status_lines() {
        let parsed = parse_name_status("M\tsrc/main.rs\nR100\told.rs\tnew.rs\n");
        assert_eq!(
            parsed,
            vec![
                ParsedNameStatus {
                    path: "src/main.rs".to_string(),
                    status: "M".to_string(),
                },
                ParsedNameStatus {
                    path: "new.rs".to_string(),
                    status: "R".to_string(),
                },
            ]
        );
    }

    #[test]
    fn merges_status_and_applies_limit() {
        let merged = merge_file_diffs(
            &[
                ParsedNumstat {
                    path: "src/main.rs".to_string(),
                    additions: 10,
                    deletions: 2,
                    is_binary: false,
                },
                ParsedNumstat {
                    path: "assets/logo.png".to_string(),
                    additions: 0,
                    deletions: 0,
                    is_binary: true,
                },
            ],
            &[
                ParsedNameStatus {
                    path: "src/main.rs".to_string(),
                    status: "M".to_string(),
                },
                ParsedNameStatus {
                    path: "assets/logo.png".to_string(),
                    status: "A".to_string(),
                },
            ],
        );

        let (limited, truncated) = apply_limit(&merged, Some(1));

        assert!(truncated);
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].status, "M");
        assert_eq!(merged[1].status, "A");
    }
}
