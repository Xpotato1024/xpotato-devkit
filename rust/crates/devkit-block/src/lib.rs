use regex::Regex;
use std::fs;
use std::path::Path;

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
}

pub fn outline_file(filepath: &Path, include_imports: bool, include_docstrings: bool) -> Result<Vec<String>, std::io::Error> {
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
                if next_line.starts_with("\"\"\"") || next_line.starts_with("'''") || next_line.starts_with("///") || next_line.starts_with("/**") || next_line.starts_with("#") {
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
    if let Some(path) = filepath {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "py" => return find_function_end_python,
                "rs" | "go" | "c" | "cpp" | "h" | "hpp" | "cc" | "js" | "jsx" | "ts" | "tsx" | "java" | "cs" | "kt" | "swift" => return find_function_end_braces,
                _ => {}
            }
        }
    }
    find_function_end_fallback
}

fn find_function_end_python(lines: &[&str], start_idx: usize) -> usize {
    let start_line = lines[start_idx].trim_end_matches('\n').trim_end_matches('\r');
    let start_indent = start_line.len() - start_line.trim_start().len();
    let mut idx = start_idx + 1;
    let mut last_content = start_idx;
    while idx < lines.len() {
        let raw = lines[idx].trim_end_matches('\n').trim_end_matches('\r');
        let stripped = raw.trim();
        if !stripped.is_empty() {
            let cur_indent = raw.len() - raw.trim_start().len();
            if cur_indent <= start_indent && !stripped.starts_with('#') && !stripped.starts_with('@') && !stripped.starts_with(')') {
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
    for idx in start_idx..lines.len() {
        for ch in lines[idx].chars() {
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

pub fn find_block_bounds(
    lines: &[&str],
    line_range: Option<&str>,
    marker: Option<&str>,
    _heading: Option<&str>,
    function: Option<&str>,
    filepath: Option<&Path>,
) -> Result<(usize, usize), String> {
    if let Some(r) = line_range {
        let parts: Vec<&str> = r.split('-').collect();
        if parts.len() != 2 { return Err("Invalid line range".to_string()); }
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

    if let Some(f) = function {
        let mut start_idx = None;
        for (i, line) in lines.iter().enumerate() {
            for pattern in FUNCTION_PATTERNS.iter() {
                if let Some(cap) = pattern.captures(line) {
                    if let Some(m) = cap.get(1) {
                        if m.as_str() == f && start_idx.is_none() {
                            start_idx = Some(i);
                        }
                    }
                }
            }
            if start_idx.is_some() { break; }
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

    let (start, end) = find_block_bounds(&raw_lines, line_range, marker, heading, function, Some(filepath))?;
    Ok(raw_lines[start..end].join(""))
}

pub fn extract_context(
    filepath: &Path,
    function: &str,
    margin: usize,
) -> Result<String, String> {
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

    let (start, end) = find_block_bounds(&raw_lines, None, None, None, Some(function), Some(filepath))?;

    let ctx_start = start.saturating_sub(margin);
    let ctx_end = (end + margin).min(plain.len());

    let mut result_lines = Vec::new();
    result_lines.push(format!("--- {} L{}-{} ---", filepath.display(), ctx_start + 1, ctx_end));
    for i in ctx_start..ctx_end {
        result_lines.push(format!("{}: {}", i + 1, plain[i]));
    }
    result_lines.push("".to_string());
    Ok(result_lines.join("\n"))
}
