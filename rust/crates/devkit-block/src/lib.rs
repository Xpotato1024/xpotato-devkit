use regex::Regex;
use std::fs;
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

lazy_static::lazy_static! {
    static ref FUNCTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap(),
        Regex::new(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)\b").unwrap(),
        Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]").unwrap(),
        Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|mod|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b").unwrap(),
        Regex::new(r"^\s*impl(?:<[^>]+>)?\s+([A-Za-z_][A-Za-z0-9_]*)\b").unwrap(),
        Regex::new(r"^\s*func\s+(?:\([^)]+\)\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap(),
        Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*[<(]").unwrap(),
        Regex::new(r"^\s*(?:export\s+)?class\s+([A-Za-z_$][A-Za-z0-9_$]*)\b").unwrap(),
    ];
    static ref IMPORT_RE: Regex = Regex::new(r#"^\s*(?:import |from \S+ import |use |require\(|#include |const \S+ = require\(|package )"#).unwrap();
    static ref DECORATOR_RE: Regex = Regex::new(r"^\s*@").unwrap();
    static ref DOCSTRING_OPEN_RE: Regex = Regex::new(r#"^\s*(?:"""|'''|///|/\*\*)"#).unwrap();
    static ref MARKDOWN_HEADING_RE: Regex = Regex::new(r"^(#{1,6})\s+(.+?)\s*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingEntry {
    pub line: usize,
    pub level: usize,
    pub text: String,
    pub slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionEntry {
    pub line: usize,
    pub name: String,
}

pub fn outline_file(
    filepath: &Path,
    include_imports: bool,
    include_docstrings: bool,
) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(filepath)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    if include_imports {
        for (i, line) in lines.iter().enumerate() {
            if IMPORT_RE.is_match(line) {
                result.push(format!("L{}: {}", i + 1, line.trim_end()));
            }
        }
        if !result.is_empty() {
            result.push("".to_string());
        }
    }

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let mut matched = false;
        for pattern in FUNCTION_PATTERNS.iter() {
            if pattern.is_match(line) {
                matched = true;
                break;
            }
        }

        if matched {
            let mut decorators = Vec::new();
            let mut j = i as isize - 1;
            while j >= 0 && DECORATOR_RE.is_match(lines[j as usize]) {
                decorators.insert(0, format!("L{}: {}", j + 1, lines[j as usize].trim_end()));
                j -= 1;
            }
            result.extend(decorators);
            result.push(format!("L{}: {}", i + 1, line.trim_end()));

            if include_docstrings && i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                // Simple workaround for raw string bug, just use starts_with
                if next_line.starts_with("\"\"\"")
                    || next_line.starts_with("'''")
                    || next_line.starts_with("///")
                    || next_line.starts_with("/**")
                    || next_line.starts_with("#")
                {
                    let mut doc_line = next_line;
                    if doc_line.starts_with("\"\"\"") || doc_line.starts_with("'''") {
                        doc_line = doc_line.trim_matches('"').trim_matches('\'').trim();
                    } else if doc_line.starts_with("///") {
                        doc_line = doc_line.trim_start_matches("///").trim();
                    }
                    if !doc_line.is_empty() {
                        result.push(format!("L{}:     # {}", i + 2, doc_line));
                    }
                }
            }
        }
        i += 1;
    }

    Ok(result)
}

fn detect_end_strategy(filepath: Option<&Path>) -> fn(&[&str], usize) -> usize {
    if let Some(path) = filepath
        && let Some(ext) = path.extension().and_then(|e| e.to_str())
    {
        match ext.to_lowercase().as_str() {
            "py" => return find_function_end_python,
            "rs" | "go" | "c" | "cpp" | "h" | "hpp" | "cc" | "js" | "jsx" | "ts" | "tsx"
            | "java" | "cs" | "kt" | "swift" => return find_function_end_braces,
            _ => {}
        }
    }
    find_function_end_fallback
}

fn find_function_end_python(lines: &[&str], start_idx: usize) -> usize {
    let start_line = lines[start_idx]
        .trim_end_matches('\n')
        .trim_end_matches('\r');
    let start_indent = start_line.len() - start_line.trim_start().len();
    let mut idx = start_idx + 1;
    let mut last_content = start_idx;
    while idx < lines.len() {
        let raw = lines[idx].trim_end_matches('\n').trim_end_matches('\r');
        let stripped = raw.trim();
        if !stripped.is_empty() {
            let cur_indent = raw.len() - raw.trim_start().len();
            if cur_indent <= start_indent
                && !stripped.starts_with('#')
                && !stripped.starts_with('@')
                && !stripped.starts_with(')')
            {
                break;
            }
            last_content = idx;
        }
        idx += 1;
    }
    last_content + 1
}

fn find_function_end_braces(lines: &[&str], start_idx: usize) -> usize {
    let mut depth = 0;
    let mut found_open = false;
    for (idx, line) in lines.iter().enumerate().skip(start_idx) {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                found_open = true;
            } else if ch == '}' {
                depth -= 1;
                if found_open && depth == 0 {
                    return idx + 1;
                }
            }
        }
    }
    lines.len()
}

fn find_function_end_fallback(lines: &[&str], start_idx: usize) -> usize {
    let mut idx = start_idx + 1;
    while idx < lines.len() {
        if lines[idx].trim().is_empty() {
            break;
        }
        idx += 1;
    }
    idx
}

pub fn list_markdown_headings(filepath: &Path) -> Result<Vec<HeadingEntry>, String> {
    let content = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if let Some(caps) = MARKDOWN_HEADING_RE.captures(line) {
            let markers = caps.get(1).unwrap().as_str();
            let text = caps.get(2).unwrap().as_str().trim().to_string();
            result.push(HeadingEntry {
                line: idx + 1,
                level: markers.len(),
                slug: slugify_heading(&text),
                text,
            });
        }
    }

    Ok(result)
}

pub fn list_functions(filepath: &Path) -> Result<Vec<FunctionEntry>, String> {
    let content = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        for pattern in FUNCTION_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(line)
                && let Some(matched) = caps.get(1)
            {
                result.push(FunctionEntry {
                    line: idx + 1,
                    name: matched.as_str().to_string(),
                });
                break;
            }
        }
    }

    Ok(result)
}

fn slugify_heading(text: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in text.nfkd().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !previous_dash {
            if !slug.is_empty() {
                slug.push('-');
                previous_dash = true;
            }
        }
    }

    slug.trim_matches('-').to_string()
}

fn find_heading_bounds(
    lines: &[&str],
    heading: &str,
    heading_exact: bool,
) -> Result<(usize, usize), String> {
    let requested = heading.trim().trim_start_matches('#').trim();
    let mut matches = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        if let Some(caps) = MARKDOWN_HEADING_RE.captures(line.trim_end_matches(['\n', '\r'])) {
            let text = caps.get(2).unwrap().as_str().trim();
            let is_match = if heading_exact {
                text == requested
            } else {
                text.contains(requested) || line.contains(heading)
            };
            if is_match {
                let level = caps.get(1).unwrap().as_str().len();
                matches.push((idx, level, line.to_string(), text.to_string()));
            }
        }
    }

    if matches.is_empty() {
        let available = lines
            .iter()
            .filter_map(|line| {
                MARKDOWN_HEADING_RE
                    .captures(line.trim_end_matches(['\n', '\r']))
                    .and_then(|caps| caps.get(2).map(|m| m.as_str().trim().to_string()))
            })
            .collect::<Vec<_>>();
        if let Some(suggestion) = available
            .iter()
            .find(|candidate| levenshtein(candidate, requested) <= 2)
        {
            return Err(format!(
                "Heading '{}' not found. Did you mean '{}'? ",
                heading, suggestion
            ));
        }
        return Err(format!("Heading '{}' not found", heading));
    }

    if matches.len() > 1 {
        let options = matches
            .iter()
            .map(|(_, _, line, _)| line.trim().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Heading '{}' matched multiple sections. Please disambiguate: {}",
            heading, options
        ));
    }

    let (start, level, _, _) = &matches[0];
    let mut end = lines.len();
    for (idx, line) in lines.iter().enumerate().skip(start + 1) {
        if let Some(caps) = MARKDOWN_HEADING_RE.captures(line.trim_end_matches(['\n', '\r']))
            && caps.get(1).unwrap().as_str().len() <= *level
        {
            end = idx;
            break;
        }
    }

    Ok((*start, end))
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs = (0..=b.chars().count()).collect::<Vec<_>>();
    for (i, ca) in a.chars().enumerate() {
        let mut previous = costs[0];
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let temp = costs[j + 1];
            costs[j + 1] = if ca == cb {
                previous
            } else {
                1 + previous.min(costs[j]).min(costs[j + 1])
            };
            previous = temp;
        }
    }
    *costs.last().unwrap()
}

pub fn find_block_bounds(
    lines: &[&str],
    line_range: Option<&str>,
    marker: Option<&str>,
    heading: Option<&str>,
    function: Option<&str>,
    filepath: Option<&Path>,
    heading_exact: bool,
) -> Result<(usize, usize), String> {
    if let Some(r) = line_range {
        let parts: Vec<&str> = r.split('-').collect();
        if parts.len() != 2 {
            return Err("Invalid line range".to_string());
        }
        let start: usize = parts[0].parse().map_err(|_| "Invalid line range")?;
        let end: usize = parts[1].parse().map_err(|_| "Invalid line range")?;
        return Ok((start.saturating_sub(1), end));
    }

    if let Some(m) = marker {
        let mut start_idx = None;
        let mut end_idx = None;
        for (i, line) in lines.iter().enumerate() {
            if line.contains(m) {
                if start_idx.is_none() {
                    start_idx = Some(i);
                } else {
                    end_idx = Some(i + 1);
                    break;
                }
            }
        }
        let start = start_idx.ok_or_else(|| format!("Marker '{}' not found", m))?;
        let end = end_idx.unwrap_or(lines.len());
        return Ok((start, end));
    }

    if let Some(h) = heading {
        return find_heading_bounds(lines, h, heading_exact);
    }

    if let Some(f) = function {
        let mut start_idx = None;
        for (i, line) in lines.iter().enumerate() {
            for pattern in FUNCTION_PATTERNS.iter() {
                if let Some(cap) = pattern.captures(line)
                    && let Some(m) = cap.get(1)
                    && m.as_str() == f
                    && start_idx.is_none()
                {
                    start_idx = Some(i);
                }
            }
            if start_idx.is_some() {
                break;
            }
        }
        let start = start_idx.ok_or_else(|| format!("Function/Class '{}' not found", f))?;
        let strategy = detect_end_strategy(filepath);
        let end = strategy(lines, start);
        return Ok((start, end));
    }

    Err("Must specify line_range, marker, heading, or function.".to_string())
}

pub fn extract_block(
    filepath: &Path,
    line_range: Option<&str>,
    marker: Option<&str>,
    heading: Option<&str>,
    function: Option<&str>,
    heading_exact: bool,
) -> Result<String, String> {
    let content = fs::read_to_string(filepath).map_err(|e| e.to_string())?;

    let mut raw_lines = Vec::new();
    let mut last = 0;
    for (i, _) in content.match_indices('\n') {
        raw_lines.push(&content[last..=i]);
        last = i + 1;
    }
    if last < content.len() {
        raw_lines.push(&content[last..]);
    }

    let (start, end) = find_block_bounds(
        &raw_lines,
        line_range,
        marker,
        heading,
        function,
        Some(filepath),
        heading_exact,
    )?;
    Ok(raw_lines[start..end].join(""))
}

pub fn extract_context(filepath: &Path, function: &str, margin: usize) -> Result<String, String> {
    let content = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    let plain: Vec<&str> = content.lines().collect();

    let mut raw_lines = Vec::new();
    let mut last = 0;
    for (i, _) in content.match_indices('\n') {
        raw_lines.push(&content[last..=i]);
        last = i + 1;
    }
    if last < content.len() {
        raw_lines.push(&content[last..]);
    }

    let (start, end) = find_block_bounds(
        &raw_lines,
        None,
        None,
        None,
        Some(function),
        Some(filepath),
        false,
    )?;

    let ctx_start = start.saturating_sub(margin);
    let ctx_end = (end + margin).min(plain.len());

    let mut result_lines = Vec::new();
    result_lines.push(format!(
        "--- {} L{}-{} ---",
        filepath.display(),
        ctx_start + 1,
        ctx_end
    ));
    for (i, line) in plain.iter().enumerate().take(ctx_end).skip(ctx_start) {
        result_lines.push(format!("{}: {}", i + 1, line));
    }
    result_lines.push("".to_string());
    Ok(result_lines.join("\n"))
}

pub fn replace_block(
    filepath: &Path,
    replacement: &str,
    line_range: Option<&str>,
    marker: Option<&str>,
    heading: Option<&str>,
    function: Option<&str>,
    dry_run: bool,
    heading_exact: bool,
) -> Result<(String, String), String> {
    let content = fs::read_to_string(filepath).map_err(|e| e.to_string())?;

    let mut raw_lines = Vec::new();
    let mut last = 0;
    for (i, _) in content.match_indices('\n') {
        raw_lines.push(&content[last..=i]);
        last = i + 1;
    }
    if last < content.len() {
        raw_lines.push(&content[last..]);
    }

    let (start, end) = find_block_bounds(
        &raw_lines,
        line_range,
        marker,
        heading,
        function,
        Some(filepath),
        heading_exact,
    )?;

    let old_block = raw_lines[start..end].join("");

    let mut new_repl = replacement.to_string();
    if !new_repl.is_empty() && !new_repl.ends_with('\n') {
        new_repl.push('\n');
    }

    let mut new_lines = Vec::new();
    new_lines.extend_from_slice(&raw_lines[..start]);
    new_lines.push(&new_repl);
    new_lines.extend_from_slice(&raw_lines[end..]);

    if !dry_run {
        fs::write(filepath, new_lines.join("")).map_err(|e| e.to_string())?;
    }

    Ok((old_block, new_repl))
}

pub fn diff_preview(old_block: &str, new_block: &str, filepath: &Path) -> String {
    use similar::TextDiff;
    let diff = TextDiff::from_lines(old_block, new_block);
    let fname = filepath.display().to_string();
    diff.unified_diff()
        .header(&fname, &fname)
        .context_radius(3)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("devkit-block-test-{}", unique));
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
    fn lists_markdown_headings_with_slug() {
        let temp = TempDir::new();
        let file = temp.path.join("doc.md");
        fs::write(&file, "# Hello World\n\n## Install Guide\n").unwrap();

        let entries = list_markdown_headings(&file).unwrap();
        assert_eq!(entries[0].slug, "hello-world");
        assert_eq!(entries[1].slug, "install-guide");
    }

    #[test]
    fn lists_functions_for_rust_items() {
        let temp = TempDir::new();
        let file = temp.path.join("lib.rs");
        fs::write(
            &file,
            "pub fn hello() {}\n\nstruct Point {}\n\nimpl Point {\n    fn new() -> Self { Self {} }\n}\n",
        )
        .unwrap();

        let entries = list_functions(&file).unwrap();
        assert_eq!(entries[0].name, "hello");
        assert_eq!(entries[1].name, "Point");
        assert_eq!(entries[2].name, "Point");
    }

    #[test]
    fn extracts_heading_block() {
        let temp = TempDir::new();
        let file = temp.path.join("doc.md");
        fs::write(
            &file,
            "# Intro\nintro\n\n## Install Guide\nguide\n\n## Notes\nnotes\n",
        )
        .unwrap();

        let content = extract_block(&file, None, None, Some("Install Guide"), None, true).unwrap();
        assert!(content.contains("## Install Guide"));
        assert!(content.contains("guide"));
        assert!(!content.contains("## Notes"));
    }

    #[test]
    fn heading_lookup_suggests_close_match() {
        let temp = TempDir::new();
        let file = temp.path.join("doc.md");
        fs::write(&file, "# Intro\n\n## Install Guide\ncontent\n").unwrap();

        let error = extract_block(&file, None, None, Some("Install Gude"), None, true).unwrap_err();
        assert!(error.contains("Did you mean"));
    }
}
