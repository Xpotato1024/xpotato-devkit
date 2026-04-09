use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

lazy_static::lazy_static! {
    static ref HUNK_RE: Regex = Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").unwrap();
    static ref DIFF_FILE_RE: Regex = Regex::new(r"^(?:---|\+\+\+) [ab]/(.+)").unwrap();
    static ref PATCH_FAILED_RE: Regex = Regex::new(r"^error: patch failed: (.+?):(\d+)$").unwrap();
    static ref TARGET_MISSING_RE: Regex = Regex::new(r"^error: (.+?): No such file or directory$").unwrap();
}

pub const CLASSIFICATION_CLEAN: &str = "clean";
pub const CLASSIFICATION_INVALID_PATCH_INPUT: &str = "invalid_patch_input";
pub const CLASSIFICATION_TARGET_MISSING: &str = "target_missing";
pub const CLASSIFICATION_CONTEXT_MISMATCH: &str = "context_mismatch";
pub const CLASSIFICATION_ALREADY_APPLIED: &str = "already_applied_or_reversed";
pub const CLASSIFICATION_UNKNOWN: &str = "unknown";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HunkInfo {
    pub file: String,
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub header: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PatchHunkDiagnostic {
    pub file: String,
    pub header: String,
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub status: String,
    pub classification: String,
    pub inferred: bool,
    pub message: Option<String>,
    pub search_context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PatchFileDiagnostic {
    pub path: String,
    pub status: String,
    pub classification: String,
    pub inferred: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PatchDiagnostic {
    pub success: bool,
    pub total_hunks: usize,
    pub applied_hunks: usize,
    pub failed_hunks: usize,
    pub classification: String,
    pub inferred: bool,
    pub errors: Vec<String>,
    pub affected_files: Vec<String>,
    pub files: Vec<PatchFileDiagnostic>,
    pub hunks: Vec<PatchHunkDiagnostic>,
    pub recommended_next_steps: Vec<String>,
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

        match self.classification.as_str() {
            CLASSIFICATION_INVALID_PATCH_INPUT => {
                "FAIL: invalid patch input (no valid hunks could be diagnosed)".to_string()
            }
            CLASSIFICATION_TARGET_MISSING => format!(
                "FAIL: patch target is missing ({})",
                self.affected_display()
            ),
            CLASSIFICATION_ALREADY_APPLIED => format!(
                "FAIL: patch appears already applied or reversed ({})",
                self.affected_display()
            ),
            CLASSIFICATION_CONTEXT_MISMATCH => format!(
                "FAIL: patch context mismatch ({} of {} hunk(s) failed; files: {})",
                self.failed_hunks.max(1),
                self.total_hunks,
                self.affected_display()
            ),
            _ => format!(
                "FAIL: patch diagnosis failed ({} of {} hunk(s) failed; files: {})",
                self.failed_hunks.max(1),
                self.total_hunks,
                self.affected_display()
            ),
        }
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
            format!("Patch FAILED: {}", self.classification_label()),
            format!("Affected files: {}", self.affected_display()),
        ];

        if !self.hunks.is_empty() {
            for hunk in self
                .hunks
                .iter()
                .filter(|entry| entry.status == "failed")
                .take(3)
            {
                let mut detail = format!(
                    "  - {} @ {} ({})",
                    hunk.file, hunk.header, hunk.classification
                );
                if let Some(message) = &hunk.message {
                    detail.push_str(&format!(": {message}"));
                }
                lines.push(detail);
                if !hunk.search_context.is_empty() {
                    lines.push(format!(
                        "    expected context: {}",
                        hunk.search_context.join(" | ")
                    ));
                }
            }
        }

        for err in self.errors.iter().take(5) {
            lines.push(format!("  - {}", err));
        }

        if self.errors.len() > 5 {
            lines.push(format!(
                "  - ... and {} more error(s)",
                self.errors.len() - 5
            ));
        }

        for step in self.recommended_next_steps.iter().take(3) {
            lines.push(format!("  -> {}", step));
        }

        lines.join("\n")
    }

    fn affected_display(&self) -> String {
        if self.affected_files.is_empty() {
            "unknown".to_string()
        } else {
            self.affected_files.join(", ")
        }
    }

    fn classification_label(&self) -> &'static str {
        match self.classification.as_str() {
            CLASSIFICATION_INVALID_PATCH_INPUT => "invalid patch input",
            CLASSIFICATION_TARGET_MISSING => "target file missing",
            CLASSIFICATION_ALREADY_APPLIED => "already applied or reversed",
            CLASSIFICATION_CONTEXT_MISMATCH => "context mismatch",
            _ => "unknown patch failure",
        }
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

#[derive(Debug, Clone)]
struct FailureEvent {
    file: Option<String>,
    line: Option<usize>,
    classification: String,
    inferred: bool,
    message: Option<String>,
    search_context: Vec<String>,
}

#[derive(Debug)]
struct GitApplyRun {
    success: bool,
    stdout: String,
    stderr: String,
}

pub fn apply_patch(
    patch_file: &Path,
    dry_run: bool,
    verbose: bool,
    reject: bool,
) -> PatchDiagnostic {
    let patch_text = fs::read_to_string(patch_file).unwrap_or_default();
    let (hunks, affected_files) = parse_patch_hunks(&patch_text);
    let primary = run_git_apply(patch_file, dry_run, verbose, reject);
    let reverse_success = if !primary.success && !hunks.is_empty() {
        run_git_apply_reverse_check(patch_file)
    } else {
        false
    };
    build_diagnostic(&patch_text, hunks, affected_files, primary, reverse_success)
}

pub fn diagnose_patch(patch_file: &Path) -> PatchDiagnostic {
    apply_patch(patch_file, true, true, false)
}

fn build_diagnostic(
    patch_text: &str,
    hunks: Vec<HunkInfo>,
    affected_files: Vec<String>,
    run: GitApplyRun,
    reverse_success: bool,
) -> PatchDiagnostic {
    let total_hunks = hunks.len();
    let mut errors = combined_output_lines(&run.stdout, &run.stderr);
    let has_patch_markers = patch_text.contains("diff --git")
        || patch_text.contains("--- ")
        || patch_text.contains("+++ ");

    if run.success {
        let files = affected_files
            .iter()
            .map(|path| PatchFileDiagnostic {
                path: path.clone(),
                status: "applies".to_string(),
                classification: CLASSIFICATION_CLEAN.to_string(),
                inferred: false,
                message: None,
            })
            .collect::<Vec<_>>();
        let hunks = hunks
            .into_iter()
            .map(|hunk| PatchHunkDiagnostic {
                file: hunk.file,
                header: hunk.header,
                old_start: hunk.old_start,
                old_count: hunk.old_count,
                new_start: hunk.new_start,
                new_count: hunk.new_count,
                status: "applies".to_string(),
                classification: CLASSIFICATION_CLEAN.to_string(),
                inferred: false,
                message: None,
                search_context: Vec::new(),
            })
            .collect::<Vec<_>>();
        return PatchDiagnostic {
            success: true,
            total_hunks,
            applied_hunks: total_hunks,
            failed_hunks: 0,
            classification: CLASSIFICATION_CLEAN.to_string(),
            inferred: false,
            errors,
            affected_files,
            files,
            hunks,
            recommended_next_steps: Vec::new(),
        };
    }

    if errors.is_empty() {
        errors.push("git apply returned failure without diagnostic output".to_string());
    }

    let mut events = parse_failure_events(&errors);
    let classification = if !has_patch_markers || total_hunks == 0 {
        CLASSIFICATION_INVALID_PATCH_INPUT.to_string()
    } else if reverse_success {
        for event in &mut events {
            event.classification = CLASSIFICATION_ALREADY_APPLIED.to_string();
            event.inferred = true;
        }
        CLASSIFICATION_ALREADY_APPLIED.to_string()
    } else if events
        .iter()
        .any(|event| event.classification == CLASSIFICATION_TARGET_MISSING)
    {
        CLASSIFICATION_TARGET_MISSING.to_string()
    } else if events
        .iter()
        .any(|event| event.classification == CLASSIFICATION_CONTEXT_MISMATCH)
    {
        CLASSIFICATION_CONTEXT_MISMATCH.to_string()
    } else if events
        .iter()
        .any(|event| event.classification == CLASSIFICATION_INVALID_PATCH_INPUT)
    {
        CLASSIFICATION_INVALID_PATCH_INPUT.to_string()
    } else {
        CLASSIFICATION_UNKNOWN.to_string()
    };

    let files = build_file_diagnostics(&affected_files, &events, &classification);
    let hunks = build_hunk_diagnostics(hunks, &events, &classification);
    let failed_hunks = hunks
        .iter()
        .filter(|entry| entry.status == "failed")
        .count();
    let inferred = reverse_success || events.iter().any(|event| event.inferred);

    PatchDiagnostic {
        success: false,
        total_hunks,
        applied_hunks: 0,
        failed_hunks: failed_hunks.max(usize::from(total_hunks > 0)),
        classification: classification.clone(),
        inferred,
        errors,
        affected_files: if affected_files.is_empty() {
            files
                .iter()
                .map(|entry| entry.path.clone())
                .collect::<Vec<_>>()
        } else {
            affected_files
        },
        files,
        hunks,
        recommended_next_steps: recommended_next_steps(&classification),
    }
}

fn run_git_apply(patch_file: &Path, dry_run: bool, verbose: bool, reject: bool) -> GitApplyRun {
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
    GitApplyRun {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn run_git_apply_reverse_check(patch_file: &Path) -> bool {
    Command::new("git")
        .arg("apply")
        .arg("--check")
        .arg("--reverse")
        .arg(patch_file)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn combined_output_lines(stdout: &str, stderr: &str) -> Vec<String> {
    let combined = if !stderr.trim().is_empty() {
        format!("{}\n{}", stdout.trim(), stderr.trim())
    } else {
        stdout.trim().to_string()
    };
    combined
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_failure_events(errors: &[String]) -> Vec<FailureEvent> {
    let mut events = Vec::new();
    let mut pending_search_context = Vec::new();
    let mut collecting_search_context = false;

    for line in errors {
        if line == "error: while searching for:" {
            collecting_search_context = true;
            pending_search_context.clear();
            continue;
        }

        if collecting_search_context {
            if line.starts_with("error:") {
                collecting_search_context = false;
            } else {
                pending_search_context.push(line.to_string());
                continue;
            }
        }

        if let Some(caps) = PATCH_FAILED_RE.captures(line) {
            events.push(FailureEvent {
                file: caps.get(1).map(|value| value.as_str().to_string()),
                line: caps.get(2).and_then(|value| value.as_str().parse().ok()),
                classification: CLASSIFICATION_CONTEXT_MISMATCH.to_string(),
                inferred: false,
                message: Some(line.clone()),
                search_context: std::mem::take(&mut pending_search_context),
            });
            continue;
        }

        if let Some(caps) = TARGET_MISSING_RE.captures(line) {
            events.push(FailureEvent {
                file: caps.get(1).map(|value| value.as_str().to_string()),
                line: None,
                classification: CLASSIFICATION_TARGET_MISSING.to_string(),
                inferred: false,
                message: Some(line.clone()),
                search_context: Vec::new(),
            });
            continue;
        }

        if line.contains("No valid patches in input") {
            events.push(FailureEvent {
                file: None,
                line: None,
                classification: CLASSIFICATION_INVALID_PATCH_INPUT.to_string(),
                inferred: false,
                message: Some(line.clone()),
                search_context: Vec::new(),
            });
        }
    }

    events
}

fn build_file_diagnostics(
    affected_files: &[String],
    events: &[FailureEvent],
    classification: &str,
) -> Vec<PatchFileDiagnostic> {
    let mut files = if affected_files.is_empty() {
        Vec::new()
    } else {
        affected_files.to_vec()
    };
    for event in events {
        if let Some(path) = &event.file
            && !files.contains(path)
        {
            files.push(path.clone());
        }
    }
    if files.is_empty() {
        files.push("unknown".to_string());
    }

    files
        .into_iter()
        .map(|path| {
            let matching_event = events
                .iter()
                .find(|event| event.file.as_deref() == Some(path.as_str()));
            PatchFileDiagnostic {
                path,
                status: "failed".to_string(),
                classification: matching_event
                    .map(|event| event.classification.clone())
                    .unwrap_or_else(|| classification.to_string()),
                inferred: matching_event.is_none()
                    || matching_event.is_some_and(|event| event.inferred),
                message: matching_event.and_then(|event| event.message.clone()),
            }
        })
        .collect()
}

fn build_hunk_diagnostics(
    hunks: Vec<HunkInfo>,
    events: &[FailureEvent],
    classification: &str,
) -> Vec<PatchHunkDiagnostic> {
    let failing_indexes = explicit_failure_indexes(&hunks, events);
    hunks
        .into_iter()
        .enumerate()
        .map(|(index, hunk)| {
            let matching_event = events.iter().find(|event| {
                event.file.as_deref() == Some(hunk.file.as_str())
                    && event.line == Some(hunk.old_start)
            });
            let is_explicit_failure = failing_indexes.contains(&index);
            let status = if is_explicit_failure
                || (!events.is_empty() && classification != CLASSIFICATION_CONTEXT_MISMATCH)
            {
                "failed"
            } else if events.is_empty() {
                "unknown"
            } else {
                "unchecked"
            };
            PatchHunkDiagnostic {
                file: hunk.file,
                header: hunk.header,
                old_start: hunk.old_start,
                old_count: hunk.old_count,
                new_start: hunk.new_start,
                new_count: hunk.new_count,
                status: status.to_string(),
                classification: if is_explicit_failure {
                    matching_event
                        .map(|event| event.classification.clone())
                        .unwrap_or_else(|| classification.to_string())
                } else if events.is_empty() {
                    CLASSIFICATION_UNKNOWN.to_string()
                } else {
                    classification.to_string()
                },
                inferred: !is_explicit_failure
                    || matching_event.is_some_and(|event| event.inferred),
                message: matching_event.and_then(|event| event.message.clone()),
                search_context: matching_event
                    .map(|event| event.search_context.clone())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

fn explicit_failure_indexes(hunks: &[HunkInfo], events: &[FailureEvent]) -> Vec<usize> {
    let mut indexes = Vec::new();
    for event in events {
        let Some(file) = event.file.as_deref() else {
            continue;
        };
        let Some(line) = event.line else {
            continue;
        };
        let mut best_match = None;
        let mut best_distance = usize::MAX;
        for (index, hunk) in hunks.iter().enumerate() {
            if hunk.file != file {
                continue;
            }
            let distance = hunk.old_start.abs_diff(line);
            if distance < best_distance {
                best_distance = distance;
                best_match = Some(index);
            }
        }
        if let Some(index) = best_match
            && !indexes.contains(&index)
        {
            indexes.push(index);
        }
    }
    indexes
}

fn recommended_next_steps(classification: &str) -> Vec<String> {
    match classification {
        CLASSIFICATION_INVALID_PATCH_INPUT => vec![
            "regenerate the patch as a unified diff instead of passing a source file".to_string(),
            "run `devkit patch diagnose --patch-file <patch>` again before applying".to_string(),
        ],
        CLASSIFICATION_TARGET_MISSING => vec![
            "confirm the target path with `devkit tree` before regenerating the patch".to_string(),
            "restore or create the target file, then re-run `devkit patch diagnose`".to_string(),
        ],
        CLASSIFICATION_ALREADY_APPLIED => vec![
            "inspect the current file state before reapplying this patch".to_string(),
            "if you meant to undo the change, apply the patch in reverse explicitly".to_string(),
        ],
        CLASSIFICATION_CONTEXT_MISMATCH => vec![
            "refresh local context with `devkit block context` or `devkit block extract`"
                .to_string(),
            "regenerate the patch from the current file content before applying".to_string(),
        ],
        _ => vec![
            "inspect the raw git errors and affected file list before retrying".to_string(),
            "re-run `devkit patch diagnose --json` if another tool will classify the failure"
                .to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brief_summary_for_success_is_single_line() {
        let diag = PatchDiagnostic {
            success: true,
            total_hunks: 2,
            applied_hunks: 2,
            failed_hunks: 0,
            classification: CLASSIFICATION_CLEAN.to_string(),
            inferred: false,
            errors: vec![],
            affected_files: vec!["src/lib.rs".to_string(), "README.md".to_string()],
            files: vec![],
            hunks: vec![],
            recommended_next_steps: vec![],
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
            applied_hunks: 0,
            failed_hunks: 2,
            classification: CLASSIFICATION_CONTEXT_MISMATCH.to_string(),
            inferred: false,
            errors: vec!["hunk 1 failed".to_string()],
            affected_files: vec!["src/main.rs".to_string()],
            files: vec![],
            hunks: vec![],
            recommended_next_steps: vec![],
        };

        assert_eq!(
            diag.brief_summary(),
            "FAIL: patch context mismatch (2 of 3 hunk(s) failed; files: src/main.rs)"
        );
    }

    #[test]
    fn brief_summary_for_invalid_patch_is_stable() {
        let diag = PatchDiagnostic {
            success: false,
            total_hunks: 0,
            applied_hunks: 0,
            failed_hunks: 0,
            classification: CLASSIFICATION_INVALID_PATCH_INPUT.to_string(),
            inferred: false,
            errors: vec!["error: No valid patches in input".to_string()],
            affected_files: vec![],
            files: vec![],
            hunks: vec![],
            recommended_next_steps: vec![],
        };

        assert_eq!(
            diag.brief_summary(),
            "FAIL: invalid patch input (no valid hunks could be diagnosed)"
        );
    }

    #[test]
    fn parses_failure_events_from_git_apply_output() {
        let errors = vec![
            "Checking patch sample.txt...".to_string(),
            "error: while searching for:".to_string(),
            "alpha".to_string(),
            "zeta".to_string(),
            "gamma".to_string(),
            "error: patch failed: sample.txt:1".to_string(),
            "error: sample.txt: patch does not apply".to_string(),
        ];

        let events = parse_failure_events(&errors);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].classification, CLASSIFICATION_CONTEXT_MISMATCH);
        assert_eq!(events[0].file.as_deref(), Some("sample.txt"));
        assert_eq!(events[0].line, Some(1));
        assert_eq!(events[0].search_context, vec!["alpha", "zeta", "gamma"]);
    }
}
