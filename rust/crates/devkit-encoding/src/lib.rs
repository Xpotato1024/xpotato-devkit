use glob::glob;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_IGNORE_PATTERNS: &[&str] =
    &[".git", "node_modules", "__pycache__", ".venv", "venv"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlineStyle {
    Lf,
    Crlf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingCheckResult {
    pub file: String,
    pub valid_utf8: bool,
    pub has_bom: bool,
    pub has_replacement_char: bool,
    pub has_control_chars: bool,
    pub mixed_newlines: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizeResult {
    pub file: String,
    pub changed: bool,
    pub removed_bom: bool,
    pub normalized_newlines: bool,
}

impl EncodingCheckResult {
    pub fn issue_labels(&self) -> Vec<&'static str> {
        let mut issues = Vec::new();
        if !self.valid_utf8 {
            issues.push("invalid_utf8");
        }
        if self.has_bom {
            issues.push("BOM");
        }
        if self.has_replacement_char {
            issues.push("replacement_char");
        }
        if self.has_control_chars {
            issues.push("control_chars");
        }
        if self.mixed_newlines {
            issues.push("mixed_newlines");
        }
        issues
    }

    pub fn has_issues(&self) -> bool {
        !self.valid_utf8
            || self.has_bom
            || self.has_replacement_char
            || self.has_control_chars
            || self.mixed_newlines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectInputsError {
    NoFilesMatched,
    InvalidGlob { pattern: String, error: String },
}

impl std::fmt::Display for CollectInputsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoFilesMatched => write!(f, "No files matched the input patterns."),
            Self::InvalidGlob { pattern, error } => {
                write!(f, "Invalid glob pattern '{}': {}", pattern, error)
            }
        }
    }
}

impl std::error::Error for CollectInputsError {}

pub fn resolve_ignore_patterns(configured: &[String]) -> Vec<String> {
    if configured.is_empty() {
        DEFAULT_IGNORE_PATTERNS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    } else {
        configured.to_vec()
    }
}

pub fn collect_inputs(cwd: &Path, inputs: &[String]) -> Result<Vec<PathBuf>, CollectInputsError> {
    let mut files = BTreeSet::new();
    let mut saw_non_glob = false;

    for input in inputs {
        if is_glob_pattern(input) {
            let pattern = resolve_pattern(cwd, input);
            let pattern_text = pattern.to_string_lossy().to_string();
            let matches = glob(&pattern_text).map_err(|error| CollectInputsError::InvalidGlob {
                pattern: input.clone(),
                error: error.to_string(),
            })?;

            for path in matches.flatten() {
                files.insert(path);
            }
        } else {
            saw_non_glob = true;
            files.insert(resolve_path(cwd, input));
        }
    }

    if files.is_empty() && !saw_non_glob {
        return Err(CollectInputsError::NoFilesMatched);
    }

    Ok(files.into_iter().collect())
}

pub fn should_ignore(path: &Path, ignore_patterns: &[String]) -> bool {
    let normalized_path = normalize_path(path);
    let file_name = path.file_name().and_then(|name| name.to_str());

    ignore_patterns.iter().any(|pattern| {
        let normalized_pattern = pattern.replace('\\', "/");

        path.components()
            .any(|component| component.as_os_str() == pattern.as_str())
            || file_name.is_some_and(|name| name == pattern)
            || glob::Pattern::new(&normalized_pattern)
                .map(|glob_pattern| {
                    glob_pattern.matches(&normalized_path)
                        || file_name.is_some_and(|name| glob_pattern.matches(name))
                })
                .unwrap_or(false)
    })
}

pub fn check_encoding(path: &Path, display_path: String) -> EncodingCheckResult {
    let mut result = EncodingCheckResult {
        file: display_path,
        valid_utf8: true,
        has_bom: false,
        has_replacement_char: false,
        has_control_chars: false,
        mixed_newlines: false,
        error: None,
    };

    let content_bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            result.valid_utf8 = false;
            result.error = Some(error.to_string());
            return result;
        }
    };

    result.has_bom = content_bytes.starts_with(&[0xEF, 0xBB, 0xBF]);

    let content = match std::str::from_utf8(&content_bytes) {
        Ok(text) => text,
        Err(error) => {
            result.valid_utf8 = false;
            result.error = Some(error.to_string());
            return result;
        }
    };

    result.has_replacement_char = content.contains('\u{fffd}');
    result.has_control_chars = content
        .chars()
        .any(|ch| ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t');
    result.mixed_newlines = has_mixed_newlines(content.as_bytes());

    result
}

pub fn display_path(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn normalize_encoding(
    path: &Path,
    display_path: String,
    newline_style: NewlineStyle,
    dry_run: bool,
) -> Result<NormalizeResult, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let removed_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    let content = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
    let without_bom = if removed_bom && content.starts_with('\u{feff}') {
        &content['\u{feff}'.len_utf8()..]
    } else {
        content
    };

    let lf_normalized = without_bom.replace("\r\n", "\n").replace('\r', "\n");
    let normalized = match newline_style {
        NewlineStyle::Lf => lf_normalized.clone(),
        NewlineStyle::Crlf => lf_normalized.replace('\n', "\r\n"),
    };

    let normalized_newlines = bytes != normalized.as_bytes();
    if normalized_newlines && !dry_run {
        fs::write(path, normalized.as_bytes()).map_err(|error| error.to_string())?;
    }

    Ok(NormalizeResult {
        file: display_path,
        changed: normalized_newlines,
        removed_bom,
        normalized_newlines,
    })
}

fn has_mixed_newlines(bytes: &[u8]) -> bool {
    let mut has_crlf = false;
    let mut has_lf = false;
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\n' {
            if index > 0 && bytes[index - 1] == b'\r' {
                has_crlf = true;
            } else {
                has_lf = true;
            }
        }
        if has_crlf && has_lf {
            return true;
        }
        index += 1;
    }

    false
}

fn is_glob_pattern(input: &str) -> bool {
    input.contains('*') || input.contains('?') || input.contains('[')
}

fn resolve_pattern(cwd: &Path, input: &str) -> PathBuf {
    let pattern = PathBuf::from(input);
    if pattern.is_absolute() {
        pattern
    } else {
        cwd.join(pattern)
    }
}

fn resolve_path(cwd: &Path, input: &str) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn next_temp_id() -> u64 {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "devkit-encoding-test-{}-{}-{}",
                std::process::id(),
                unique,
                next_temp_id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn detects_bom_control_chars_and_mixed_newlines() {
        let temp = TempDir::new();
        let file = temp.path.join("sample.txt");
        fs::write(&file, b"\xEF\xBB\xBFhello\r\nline\x07\n").unwrap();

        let result = check_encoding(&file, "sample.txt".to_string());

        assert!(result.valid_utf8);
        assert!(result.has_bom);
        assert!(result.has_control_chars);
        assert!(result.mixed_newlines);
        assert_eq!(
            result.issue_labels(),
            vec!["BOM", "control_chars", "mixed_newlines"]
        );
    }

    #[test]
    fn reports_invalid_utf8() {
        let temp = TempDir::new();
        let file = temp.path.join("broken.txt");
        fs::write(&file, [0xFF, 0xFE, 0x61]).unwrap();

        let result = check_encoding(&file, "broken.txt".to_string());

        assert!(!result.valid_utf8);
        assert!(result.error.is_some());
        assert_eq!(result.issue_labels(), vec!["invalid_utf8"]);
    }

    #[test]
    fn collects_globbed_and_direct_inputs() {
        let temp = TempDir::new();
        let inputs_dir = temp.path.join("inputs");
        fs::create_dir_all(&inputs_dir).unwrap();
        fs::write(inputs_dir.join("a.txt"), "a\n").unwrap();
        fs::write(inputs_dir.join("b.txt"), "b\n").unwrap();

        let inputs = vec!["inputs/*.txt".to_string(), "missing.txt".to_string()];
        let files = collect_inputs(&temp.path, &inputs).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|path| path.ends_with("a.txt")));
        assert!(files.iter().any(|path| path.ends_with("b.txt")));
        assert!(files.iter().any(|path| path.ends_with("missing.txt")));
    }

    #[test]
    fn ignores_common_directories_by_component_name() {
        let temp = TempDir::new();
        let file = temp.path.join("node_modules").join("pkg").join("index.js");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "ok\n").unwrap();

        let ignore = resolve_ignore_patterns(&[]);
        assert!(should_ignore(&file, &ignore));
    }

    #[test]
    fn normalizes_to_crlf_and_removes_bom() {
        let temp = TempDir::new();
        let file = temp.path.join("sample.txt");
        fs::write(&file, b"\xEF\xBB\xBFhello\nworld\r\n").unwrap();

        let result =
            normalize_encoding(&file, "sample.txt".to_string(), NewlineStyle::Crlf, false).unwrap();

        assert!(result.changed);
        assert!(result.removed_bom);
        assert_eq!(fs::read(&file).unwrap(), b"hello\r\nworld\r\n");
    }
}
