use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

lazy_static::lazy_static! {
    static ref HUNK_RE: Regex = Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").unwrap();
    static ref DIFF_FILE_RE: Regex = Regex::new(r"^(?:---|\+\+\+) [ab]/(.+)").unwrap();
}

#[derive(Debug)]
pub struct HunkInfo {
    pub file: String,
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub header: String,
}

#[derive(Debug)]
pub struct PatchDiagnostic {
    pub success: bool,
    pub total_hunks: usize,
    pub applied_hunks: usize,
    pub failed_hunks: usize,
    pub errors: Vec<String>,
    pub affected_files: Vec<String>,
}

impl PatchDiagnostic {
    pub fn brief_summary(&self) -> String {
        if self.success {
            return format!(
                "OK: patch applied cleanly ({} hunk(s), {} file(s))",
                self.total_hunks,
                self.affected_files.len()
            );
        }

        let affected = if self.affected_files.is_empty() {
            "unknown".to_string()
        } else {
            self.affected_files.join(", ")
        };

        format!(
            "FAIL: patch application failed ({} of {} hunk(s) failed; files: {})",
            self.failed_hunks, self.total_hunks, affected
        )
    }

    pub fn summary(&self) -> String {
        if self.success {
            return format!(
                "Patch applied cleanly ({} hunk(s), {} file(s)).",
                self.total_hunks,
                self.affected_files.len()
            );
        }

        let mut lines = vec![
            format!(
                "Patch FAILED: {}/{} hunk(s) failed.",
                self.failed_hunks, self.total_hunks
            ),
            format!(
                "Affected files: {}",
                if self.affected_files.is_empty() {
                    "unknown".to_string()
                } else {
                    self.affected_files.join(", ")
                }
            ),
        ];

        for err in self.errors.iter().take(5) {
            lines.push(format!("  - {}", err));
        }

        if self.errors.len() > 5 {
            lines.push(format!(
                "  - ... and {} more error(s)",
                self.errors.len() - 5
            ));
        }

        lines.join("\n")
    }
}

pub fn parse_patch_hunks(patch_text: &str) -> (Vec<HunkInfo>, Vec<String>) {
    let mut hunks = Vec::new();
    let mut files = Vec::new();
    let mut current_file = "unknown".to_string();

    for line in patch_text.lines() {
        if let Some(caps) = DIFF_FILE_RE.captures(line) {
            if let Some(fname) = caps.get(1) {
                let f = fname.as_str();
                if f != "/dev/null" && !files.contains(&f.to_string()) {
                    current_file = f.to_string();
                    files.push(f.to_string());
                }
            }
            continue;
        }

        if let Some(caps) = HUNK_RE.captures(line) {
            let old_start = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let old_count = caps.get(2).map_or(1, |m| m.as_str().parse().unwrap_or(1));
            let new_start = caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let new_count = caps.get(4).map_or(1, |m| m.as_str().parse().unwrap_or(1));

            hunks.push(HunkInfo {
                file: current_file.clone(),
                old_start,
                old_count,
                new_start,
                new_count,
                header: line.to_string(),
            });
        }
    }

    (hunks, files)
}

pub fn apply_patch(
    patch_file: &Path,
    dry_run: bool,
    verbose: bool,
    reject: bool,
) -> PatchDiagnostic {
    let patch_text = fs::read_to_string(patch_file).unwrap_or_default();

    let (hunks, affected_files) = parse_patch_hunks(&patch_text);

    let mut cmd = Command::new("git");
    cmd.arg("apply");
    if dry_run {
        cmd.arg("--check");
    }
    if reject {
        cmd.arg("--reject");
    }
    if verbose {
        cmd.arg("--verbose");
    }
    cmd.arg(patch_file);

    let output = cmd.output().expect("failed to execute git apply");
    let success = output.status.success();

    let mut diag = PatchDiagnostic {
        success,
        total_hunks: hunks.len(),
        applied_hunks: 0,
        failed_hunks: 0,
        errors: Vec::new(),
        affected_files,
    };

    if success {
        diag.applied_hunks = hunks.len();
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_text = if !stderr.trim().is_empty() {
            stderr.trim()
        } else {
            stdout.trim()
        };

        diag.errors = error_text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let mut fail_count = 0;
        for err in &diag.errors {
            let lower = err.to_lowercase();
            if lower.contains("patch does not apply") || lower.contains("hunk") {
                fail_count += 1;
            }
        }

        diag.failed_hunks = fail_count.max(1);
        diag.applied_hunks = hunks.len().saturating_sub(diag.failed_hunks);
    }

    diag
}

pub fn diagnose_patch(patch_file: &Path) -> PatchDiagnostic {
    apply_patch(patch_file, true, true, false)
}

#[cfg(test)]
mod tests {
    use super::PatchDiagnostic;

    #[test]
    fn brief_summary_for_success_is_single_line() {
        let diag = PatchDiagnostic {
            success: true,
            total_hunks: 2,
            applied_hunks: 2,
            failed_hunks: 0,
            errors: vec![],
            affected_files: vec!["src/lib.rs".to_string(), "README.md".to_string()],
        };

        assert_eq!(
            diag.brief_summary(),
            "OK: patch applied cleanly (2 hunk(s), 2 file(s))"
        );
    }

    #[test]
    fn brief_summary_for_failure_mentions_failed_hunks() {
        let diag = PatchDiagnostic {
            success: false,
            total_hunks: 3,
            applied_hunks: 1,
            failed_hunks: 2,
            errors: vec!["hunk 1 failed".to_string()],
            affected_files: vec!["src/main.rs".to_string()],
        };

        assert_eq!(
            diag.brief_summary(),
            "FAIL: patch application failed (2 of 3 hunk(s) failed; files: src/main.rs)"
        );
    }
}
