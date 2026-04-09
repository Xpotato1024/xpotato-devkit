use glob::Pattern;
use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchView {
    Hits,
    FilesWithMatches,
    Count,
}

#[derive(Debug, Clone)]
pub struct SearchOptions<'a> {
    pub glob: Option<&'a str>,
    pub types: &'a [String],
    pub ignore_case: bool,
    pub fixed_strings: bool,
    pub context: usize,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ContextLine {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TextSearchHit {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub preview: String,
    pub before: Vec<ContextLine>,
    pub after: Vec<ContextLine>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SymbolSearchHit {
    pub path: String,
    pub line: usize,
    pub name: String,
    pub preview: String,
    pub before: Vec<ContextLine>,
    pub after: Vec<ContextLine>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FileMatchCount {
    pub path: String,
    pub matches: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SearchSummary<T> {
    pub query: String,
    pub results: Vec<T>,
    pub total_matches: usize,
    pub matched_files: usize,
    pub skipped_binary_or_unreadable: usize,
    pub truncated: bool,
}

pub fn search_text(
    root: &Path,
    query: &str,
    options: &SearchOptions<'_>,
    view: SearchView,
) -> Result<SearchSummary<TextSearchHit>, String> {
    let matcher = compile_matcher(query, options)?;
    let candidates = collect_candidate_paths(root, options)?;
    let mut results = Vec::new();
    let mut total_matches = 0;
    let mut matched_files = 0;
    let mut skipped_binary_or_unreadable = 0;
    let mut truncated = false;

    for path in candidates {
        let relative = relative_display(root, &path);
        let content = match read_utf8_text(&path) {
            Ok(content) => content,
            Err(_) => {
                skipped_binary_or_unreadable += 1;
                continue;
            }
        };
        let lines = content.lines().collect::<Vec<_>>();
        let mut file_match_count = 0usize;

        for (index, line) in lines.iter().enumerate() {
            let line_matches = matcher.find_iter(line).collect::<Vec<_>>();
            if line_matches.is_empty() {
                continue;
            }

            for found in line_matches {
                total_matches += 1;
                file_match_count += 1;

                if matches!(view, SearchView::Hits) {
                    if reached_limit(options.limit, results.len()) {
                        truncated = true;
                        break;
                    }
                    results.push(TextSearchHit {
                        path: relative.clone(),
                        line: index + 1,
                        column: byte_offset_to_column(line, found.start()),
                        preview: (*line).to_string(),
                        before: context_before(&lines, index, options.context),
                        after: context_after(&lines, index, options.context),
                    });
                }
            }

            if truncated && matches!(view, SearchView::Hits) {
                break;
            }
        }

        if file_match_count > 0 {
            matched_files += 1;
            match view {
                SearchView::Hits => {}
                SearchView::FilesWithMatches => {
                    if reached_limit(options.limit, results.len()) {
                        truncated = true;
                        break;
                    }
                    results.push(TextSearchHit {
                        path: relative,
                        line: 0,
                        column: 0,
                        preview: String::new(),
                        before: Vec::new(),
                        after: Vec::new(),
                    });
                }
                SearchView::Count => {
                    if reached_limit(options.limit, results.len()) {
                        truncated = true;
                        break;
                    }
                    results.push(TextSearchHit {
                        path: relative,
                        line: file_match_count,
                        column: 0,
                        preview: String::new(),
                        before: Vec::new(),
                        after: Vec::new(),
                    });
                }
            }
        }

        if truncated {
            break;
        }
    }

    Ok(SearchSummary {
        query: query.to_string(),
        results,
        total_matches,
        matched_files,
        skipped_binary_or_unreadable,
        truncated,
    })
}

pub fn search_symbol(
    root: &Path,
    query: &str,
    options: &SearchOptions<'_>,
    view: SearchView,
) -> Result<SearchSummary<SymbolSearchHit>, String> {
    let matcher = compile_matcher(query, options)?;
    let candidates = collect_candidate_paths(root, options)?;
    let mut results = Vec::new();
    let mut total_matches = 0;
    let mut matched_files = 0;
    let mut skipped_binary_or_unreadable = 0;
    let mut truncated = false;

    for path in candidates {
        let relative = relative_display(root, &path);
        let content = match read_utf8_text(&path) {
            Ok(content) => content,
            Err(_) => {
                skipped_binary_or_unreadable += 1;
                continue;
            }
        };
        let symbols = devkit_block::list_functions_in_text(&content);
        if symbols.is_empty() {
            continue;
        }

        let lines = content.lines().collect::<Vec<_>>();
        let mut file_match_count = 0usize;

        for symbol in symbols {
            if !matcher.is_match(&symbol.name) {
                continue;
            }

            total_matches += 1;
            file_match_count += 1;

            if matches!(view, SearchView::Hits) {
                if reached_limit(options.limit, results.len()) {
                    truncated = true;
                    break;
                }
                let line_index = symbol.line.saturating_sub(1);
                let preview = lines
                    .get(line_index)
                    .copied()
                    .unwrap_or_default()
                    .to_string();
                results.push(SymbolSearchHit {
                    path: relative.clone(),
                    line: symbol.line,
                    name: symbol.name,
                    preview,
                    before: context_before(&lines, line_index, options.context),
                    after: context_after(&lines, line_index, options.context),
                });
            }
        }

        if file_match_count > 0 {
            matched_files += 1;
            match view {
                SearchView::Hits => {}
                SearchView::FilesWithMatches => {
                    if reached_limit(options.limit, results.len()) {
                        truncated = true;
                        break;
                    }
                    results.push(SymbolSearchHit {
                        path: relative,
                        line: 0,
                        name: String::new(),
                        preview: String::new(),
                        before: Vec::new(),
                        after: Vec::new(),
                    });
                }
                SearchView::Count => {
                    if reached_limit(options.limit, results.len()) {
                        truncated = true;
                        break;
                    }
                    results.push(SymbolSearchHit {
                        path: relative,
                        line: file_match_count,
                        name: String::new(),
                        preview: String::new(),
                        before: Vec::new(),
                        after: Vec::new(),
                    });
                }
            }
        }

        if truncated {
            break;
        }
    }

    Ok(SearchSummary {
        query: query.to_string(),
        results,
        total_matches,
        matched_files,
        skipped_binary_or_unreadable,
        truncated,
    })
}

pub fn search_text_counts(
    root: &Path,
    query: &str,
    options: &SearchOptions<'_>,
) -> Result<SearchSummary<FileMatchCount>, String> {
    search_counts(root, query, options, false)
}

pub fn search_symbol_counts(
    root: &Path,
    query: &str,
    options: &SearchOptions<'_>,
) -> Result<SearchSummary<FileMatchCount>, String> {
    search_counts(root, query, options, true)
}

fn search_counts(
    root: &Path,
    query: &str,
    options: &SearchOptions<'_>,
    symbol_mode: bool,
) -> Result<SearchSummary<FileMatchCount>, String> {
    let matcher = compile_matcher(query, options)?;
    let candidates = collect_candidate_paths(root, options)?;
    let mut results = Vec::new();
    let mut total_matches = 0;
    let mut matched_files = 0;
    let mut skipped_binary_or_unreadable = 0;
    let mut truncated = false;

    for path in candidates {
        let relative = relative_display(root, &path);
        let content = match read_utf8_text(&path) {
            Ok(content) => content,
            Err(_) => {
                skipped_binary_or_unreadable += 1;
                continue;
            }
        };

        let matches = if symbol_mode {
            devkit_block::list_functions_in_text(&content)
                .into_iter()
                .filter(|entry| matcher.is_match(&entry.name))
                .count()
        } else {
            content
                .lines()
                .map(|line| matcher.find_iter(line).count())
                .sum::<usize>()
        };

        if matches == 0 {
            continue;
        }

        matched_files += 1;
        total_matches += matches;
        if reached_limit(options.limit, results.len()) {
            truncated = true;
            break;
        }

        results.push(FileMatchCount {
            path: relative,
            matches,
        });
    }

    Ok(SearchSummary {
        query: query.to_string(),
        results,
        total_matches,
        matched_files,
        skipped_binary_or_unreadable,
        truncated,
    })
}

fn compile_matcher(query: &str, options: &SearchOptions<'_>) -> Result<Regex, String> {
    if query.is_empty() {
        return Err("search query cannot be empty".to_string());
    }

    let pattern = if options.fixed_strings {
        regex::escape(query)
    } else {
        query.to_string()
    };
    RegexBuilder::new(&pattern)
        .case_insensitive(options.ignore_case)
        .build()
        .map_err(|error| format!("invalid search pattern: {error}"))
}

fn collect_candidate_paths(
    root: &Path,
    options: &SearchOptions<'_>,
) -> Result<Vec<PathBuf>, String> {
    let glob_pattern = match options.glob {
        Some(value) => Some(Pattern::new(value).map_err(|error| format!("invalid glob: {error}"))?),
        None => None,
    };

    let mut builder = WalkBuilder::new(root);
    builder.hidden(true);
    let mut paths = builder
        .build()
        .flatten()
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(|entry| entry.into_path())
        .filter(|path| {
            let Ok(relative) = path.strip_prefix(root) else {
                return false;
            };
            matches_path(relative, glob_pattern.as_ref(), options.types)
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| relative_display(root, path));
    Ok(paths)
}

fn matches_path(rel_path: &Path, glob_pattern: Option<&Pattern>, types: &[String]) -> bool {
    if let Some(pattern) = glob_pattern {
        let normalized = normalize_path(rel_path);
        if !pattern.matches(&normalized) {
            return false;
        }
    }

    if types.is_empty() {
        return true;
    }

    let ext = rel_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    resolve_type_extensions(types)
        .iter()
        .any(|candidate| candidate == &ext)
}

fn resolve_type_extensions(types: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    for entry in types {
        let normalized = entry.trim().trim_start_matches('.').to_ascii_lowercase();
        let mapped = match normalized.as_str() {
            "rust" => vec!["rs"],
            "python" => vec!["py"],
            "markdown" => vec!["md", "markdown"],
            "json" => vec!["json"],
            "yaml" => vec!["yaml", "yml"],
            "toml" => vec!["toml"],
            "javascript" => vec!["js", "jsx", "mjs", "cjs"],
            "typescript" => vec!["ts", "tsx"],
            "shell" => vec!["sh", "bash"],
            "go" => vec!["go"],
            "java" => vec!["java"],
            "c" => vec!["c", "h"],
            "cpp" => vec!["cc", "cpp", "cxx", "hpp", "hh", "hxx"],
            other => vec![other],
        };
        for value in mapped {
            let candidate = value.to_string();
            if !values.contains(&candidate) {
                values.push(candidate);
            }
        }
    }
    values
}

fn read_utf8_text(path: &Path) -> Result<String, ()> {
    let bytes = fs::read(path).map_err(|_| ())?;
    if bytes.contains(&0) {
        return Err(());
    }
    String::from_utf8(bytes).map_err(|_| ())
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(normalize_path)
        .unwrap_or_else(|_| normalize_path(path))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn context_before(lines: &[&str], index: usize, context: usize) -> Vec<ContextLine> {
    if context == 0 {
        return Vec::new();
    }
    let start = index.saturating_sub(context);
    (start..index)
        .map(|line_index| ContextLine {
            line: line_index + 1,
            text: lines[line_index].to_string(),
        })
        .collect()
}

fn context_after(lines: &[&str], index: usize, context: usize) -> Vec<ContextLine> {
    if context == 0 {
        return Vec::new();
    }
    let end = (index + context + 1).min(lines.len());
    ((index + 1)..end)
        .map(|line_index| ContextLine {
            line: line_index + 1,
            text: lines[line_index].to_string(),
        })
        .collect()
}

fn byte_offset_to_column(line: &str, byte_offset: usize) -> usize {
    line[..byte_offset].chars().count() + 1
}

fn reached_limit(limit: Option<usize>, current_len: usize) -> bool {
    limit.is_some_and(|value| current_len >= value)
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

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "devkit-search-{name}-{}-{unique}-{}",
            std::process::id(),
            next_temp_id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn options<'a>() -> SearchOptions<'a> {
        SearchOptions {
            glob: None,
            types: &[],
            ignore_case: false,
            fixed_strings: false,
            context: 0,
            limit: None,
        }
    }

    #[test]
    fn text_search_finds_line_and_column() {
        let dir = test_dir("text-hit");
        fs::write(
            dir.join("src.rs"),
            "fn main() {\n    println!(\"hello\");\n}\n",
        )
        .unwrap();

        let summary = search_text(&dir, "println", &options(), SearchView::Hits).unwrap();

        assert_eq!(summary.total_matches, 1);
        assert_eq!(summary.matched_files, 1);
        assert_eq!(summary.results.len(), 1);
        assert_eq!(summary.results[0].path, "src.rs");
        assert_eq!(summary.results[0].line, 2);
        assert_eq!(summary.results[0].column, 5);
    }

    #[test]
    fn text_search_honors_type_and_glob_filters() {
        let dir = test_dir("filters");
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join("nested").join("one.rs"), "fn alpha() {}\n").unwrap();
        fs::write(
            dir.join("nested").join("two.py"),
            "def alpha():\n    pass\n",
        )
        .unwrap();
        let types = vec!["rust".to_string()];
        let options = SearchOptions {
            glob: Some("nested/*.rs"),
            types: &types,
            ..options()
        };

        let summary = search_text(&dir, "alpha", &options, SearchView::Hits).unwrap();

        assert_eq!(summary.total_matches, 1);
        assert_eq!(summary.results[0].path, "nested/one.rs");
    }

    #[test]
    fn text_search_count_view_aggregates_per_file() {
        let dir = test_dir("count");
        fs::write(dir.join("one.txt"), "alpha\nalpha\n").unwrap();
        fs::write(dir.join("two.txt"), "alpha\n").unwrap();

        let summary = search_text_counts(&dir, "alpha", &options()).unwrap();

        assert_eq!(summary.total_matches, 3);
        assert_eq!(summary.matched_files, 2);
        assert_eq!(summary.results[0].matches, 2);
        assert_eq!(summary.results[1].matches, 1);
    }

    #[test]
    fn symbol_search_reuses_block_heuristics() {
        let dir = test_dir("symbol");
        fs::write(
            dir.join("lib.rs"),
            "pub struct Widget;\nimpl Widget {}\npub fn helper() {}\n",
        )
        .unwrap();

        let summary = search_symbol(&dir, "Widget|helper", &options(), SearchView::Hits).unwrap();

        assert_eq!(summary.total_matches, 3);
        assert_eq!(summary.results[0].name, "Widget");
        assert_eq!(summary.results[1].name, "Widget");
        assert_eq!(summary.results[2].name, "helper");
    }

    #[test]
    fn skips_non_utf8_files() {
        let dir = test_dir("skip-binary");
        fs::write(dir.join("ok.txt"), "alpha\n").unwrap();
        fs::write(dir.join("bad.bin"), [0, 159, 146, 150]).unwrap();

        let summary = search_text(&dir, "alpha", &options(), SearchView::Hits).unwrap();

        assert_eq!(summary.total_matches, 1);
        assert_eq!(summary.skipped_binary_or_unreadable, 1);
    }
}
