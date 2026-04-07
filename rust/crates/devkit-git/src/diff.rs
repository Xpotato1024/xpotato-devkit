use serde::{Deserialize, Serialize};
use crate::run_git_command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeDesc {
    pub mode: String,
    pub description: String,
    pub refspec: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiffScope {
    pub mode: String,
    pub description: String,
    pub diff_args: Vec<String>,
    pub refspec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub scope: ScopeDesc,
    pub files: Vec<FileDiff>,
    pub total_additions: usize,
    pub total_deletions: usize,
}

pub fn build_diff_scope(
    staged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
) -> Result<DiffScope, String> {
    if commits.is_some() && (staged || base.is_some() || head.is_some()) {
        return Err("`--commits` cannot be combined with `--staged` or `--base/--head`.".to_string());
    }
    if staged && (base.is_some() || head.is_some()) {
        return Err("`--staged` cannot be combined with `--base/--head`.".to_string());
    }
    if (base.is_some() && head.is_none()) || (head.is_some() && base.is_none()) {
        return Err("`--base` and `--head` must be provided together.".to_string());
    }

    if let Some(commits) = commits {
        return Ok(DiffScope {
            mode: "commits".to_string(),
            description: format!("commit range {}", commits),
            diff_args: vec!["diff".to_string(), "--numstat".to_string(), commits.to_string()],
            refspec: Some(commits.to_string()),
        });
    }
    if let (Some(b), Some(h)) = (base, head) {
        let refspec = format!("{}...{}", b, h);
        return Ok(DiffScope {
            mode: "range".to_string(),
            description: format!("range {}...{}", b, h),
            diff_args: vec!["diff".to_string(), "--numstat".to_string(), refspec.clone()],
            refspec: Some(refspec),
        });
    }
    if staged {
        return Ok(DiffScope {
            mode: "staged".to_string(),
            description: "staged changes".to_string(),
            diff_args: vec!["diff".to_string(), "--numstat".to_string(), "--staged".to_string()],
            refspec: None,
        });
    }
    Ok(DiffScope {
        mode: "unstaged".to_string(),
        description: "unstaged changes".to_string(),
        diff_args: vec!["diff".to_string(), "--numstat".to_string()],
        refspec: None,
    })
}

pub fn summarize_diff(
    staged: bool,
    base: Option<&str>,
    head: Option<&str>,
    commits: Option<&str>,
) -> Result<DiffSummary, String> {
    let scope = build_diff_scope(staged, base, head, commits)?;
    summarize_diff_scope(&scope)
}

pub fn summarize_diff_scope(scope: &DiffScope) -> Result<DiffSummary, String> {
    let str_args: Vec<&str> = scope.diff_args.iter().map(|s| s.as_str()).collect();
    let output = match run_git_command(&str_args) {
        Ok(out) => out,
        Err(e) => return Err(e),
    };

    let mut files_changed = Vec::new();
    let mut total_additions = 0;
    let mut total_deletions = 0;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        let mut parts = line.split('\t');
        let adds = parts.next().unwrap_or("").trim();
        let dels = parts.next().unwrap_or("").trim();
        let fname = parts.next().unwrap_or("").trim();
        
        if adds.is_empty() || dels.is_empty() || fname.is_empty() { continue; }

        let adds_count = if adds != "-" { adds.parse().unwrap_or(0) } else { 0 };
        let dels_count = if dels != "-" { dels.parse().unwrap_or(0) } else { 0 };

        total_additions += adds_count;
        total_deletions += dels_count;

        files_changed.push(FileDiff {
            path: fname.to_string(),
            additions: adds_count,
            deletions: dels_count,
            is_binary: adds == "-",
        });
    }

    Ok(DiffSummary {
        scope: ScopeDesc {
            mode: scope.mode.clone(),
            description: scope.description.clone(),
            refspec: scope.refspec.clone(),
        },
        files: files_changed,
        total_additions,
        total_deletions,
    })
}
