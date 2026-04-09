use glob::Pattern;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct TreeEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: BTreeMap<String, TreeEntry>,
}

#[derive(Debug, Clone)]
pub struct TreeScanOptions<'a> {
    pub max_depth: Option<usize>,
    pub extensions: Option<&'a [String]>,
    pub dirs_only: bool,
    pub files_only: bool,
    pub use_gitignore: bool,
    pub show_hidden: bool,
    pub glob: Option<&'a str>,
    pub extra_ignore: &'a [String],
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct TreeCounts {
    pub directories: usize,
    pub files: usize,
}

impl TreeEntry {
    pub fn new(name: String, is_dir: bool, size: u64) -> Self {
        Self {
            name,
            is_dir,
            size,
            children: BTreeMap::new(),
        }
    }
}

pub fn scan_tree(root: &Path, options: &TreeScanOptions<'_>) -> TreeEntry {
    let mut builder = WalkBuilder::new(root);

    let mut ov = OverrideBuilder::new(root);
    for ig in options.extra_ignore {
        let _ = ov.add(&format!("!{}", ig));
    }
    if let Ok(o) = ov.build() {
        builder.overrides(o);
    }

    if let Some(depth) = options.max_depth {
        builder.max_depth(Some(depth));
    }

    if !options.use_gitignore {
        builder
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false);
    }
    builder.hidden(!options.show_hidden);

    let glob_pattern = options.glob.and_then(|value| Pattern::new(value).ok());
    let walker = builder.build();

    let name = root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut root_entry = TreeEntry::new(
        if name.is_empty() {
            ".".to_string()
        } else {
            name
        },
        true,
        0,
    );

    for entry in walker.flatten() {
        if entry.depth() == 0 {
            continue;
        }

        let path = entry.path();
        let is_dir = path.is_dir();
        let Ok(rel_path) = path.strip_prefix(root) else {
            continue;
        };

        if !matches_filters(rel_path, is_dir, options.extensions, glob_pattern.as_ref()) {
            continue;
        }

        if options.dirs_only && !is_dir {
            continue;
        }
        if options.files_only && is_dir {
            continue;
        }

        let mut current = &mut root_entry;
        let components: Vec<_> = rel_path.iter().collect();
        for (i, component) in components.iter().enumerate() {
            let component_name = component.to_string_lossy().to_string();
            let component_is_dir = if i == components.len() - 1 {
                is_dir
            } else {
                true
            };
            let size = if component_is_dir {
                0
            } else {
                entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
            };

            current = current
                .children
                .entry(component_name.clone())
                .or_insert_with(|| TreeEntry::new(component_name, component_is_dir, size));
        }
    }

    if options.extensions.is_some() || options.files_only || options.glob.is_some() {
        prune_empty_dirs(&mut root_entry);
    }

    root_entry
}

fn matches_filters(
    rel_path: &Path,
    is_dir: bool,
    extensions: Option<&[String]>,
    glob_pattern: Option<&Pattern>,
) -> bool {
    if !is_dir && let Some(exts) = extensions {
        let ext = rel_path
            .extension()
            .map(|value| value.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !exts
            .iter()
            .any(|value| value.trim_start_matches('.') == ext)
        {
            return false;
        }
    }

    if let Some(pattern) = glob_pattern {
        let normalized = rel_path.to_string_lossy().replace('\\', "/");
        if !pattern.matches(&normalized) {
            return false;
        }
    }

    true
}

fn prune_empty_dirs(entry: &mut TreeEntry) -> bool {
    let mut to_remove = Vec::new();
    for (key, value) in &mut entry.children {
        let has_content = prune_empty_dirs(value);
        if !has_content && value.is_dir {
            to_remove.push(key.clone());
        }
    }
    for key in to_remove {
        entry.children.remove(&key);
    }
    !entry.children.is_empty() || !entry.is_dir
}

pub fn apply_limit(entry: &mut TreeEntry, limit: usize) -> bool {
    let mut remaining = limit;
    apply_limit_inner(entry, &mut remaining)
}

fn apply_limit_inner(entry: &mut TreeEntry, remaining: &mut usize) -> bool {
    let mut truncated = false;
    let sorted_keys = sorted_child_keys(entry);
    let mut to_remove = Vec::new();

    for key in sorted_keys {
        if *remaining == 0 {
            to_remove.push(key);
            truncated = true;
            continue;
        }

        *remaining -= 1;
        if let Some(child) = entry.children.get_mut(&key)
            && child.is_dir
        {
            truncated |= apply_limit_inner(child, remaining);
        }
    }

    for key in to_remove {
        entry.children.remove(&key);
    }

    truncated
}

fn sorted_child_keys(entry: &TreeEntry) -> Vec<String> {
    let mut children: Vec<_> = entry.children.values().collect();
    children.sort_by(|left, right| {
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    children
        .into_iter()
        .map(|child| child.name.clone())
        .collect()
}

pub fn format_tree(
    entry: &TreeEntry,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    lines: &mut Vec<String>,
) {
    let display = if is_root || entry.is_dir {
        format!("{}/", entry.name)
    } else {
        format!("{} ({})", entry.name, format_size(entry.size))
    };

    if is_root {
        lines.push(display);
    } else {
        let connector = if is_last { "└── " } else { "├── " };
        lines.push(format!("{}{}{}", prefix, connector, display));
    }

    if entry.is_dir {
        let child_prefix = format!(
            "{}{}",
            prefix,
            if is_last || is_root { "    " } else { "│   " }
        );
        let sorted_keys = sorted_child_keys(entry);
        let len = sorted_keys.len();
        for (index, key) in sorted_keys.iter().enumerate() {
            if let Some(child) = entry.children.get(key) {
                format_tree(child, &child_prefix, index == len - 1, false, lines);
            }
        }
    }
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        return format!("{}B", size);
    }
    let size_f = size as f64;
    if size_f < 1024.0 * 1024.0 {
        return format!("{:.1}KB", size_f / 1024.0);
    }
    format!("{:.1}MB", size_f / (1024.0 * 1024.0))
}

pub fn count_tree(entry: &TreeEntry) -> TreeCounts {
    fn count(entry: &TreeEntry, counts: &mut TreeCounts) {
        if entry.is_dir {
            counts.directories += 1;
            for child in entry.children.values() {
                count(child, counts);
            }
        } else {
            counts.files += 1;
        }
    }

    let mut counts = TreeCounts {
        directories: 0,
        files: 0,
    };
    count(entry, &mut counts);
    counts.directories = counts.directories.saturating_sub(1);
    counts
}

pub fn tree_summary(entry: &TreeEntry) -> String {
    let counts = count_tree(entry);
    format!("{} directories, {} files", counts.directories, counts.files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
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
            "devkit-tree-{name}-{}-{unique}-{}",
            std::process::id(),
            next_temp_id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn options<'a>() -> TreeScanOptions<'a> {
        TreeScanOptions {
            max_depth: None,
            extensions: None,
            dirs_only: false,
            files_only: false,
            use_gitignore: false,
            show_hidden: false,
            glob: None,
            extra_ignore: &[],
        }
    }

    #[test]
    fn hides_hidden_files_by_default() {
        let dir = test_dir("hidden-default");
        fs::write(dir.join(".secret"), "x").unwrap();
        fs::write(dir.join("visible.txt"), "x").unwrap();

        let entry = scan_tree(&dir, &options());
        let counts = count_tree(&entry);

        assert_eq!(counts.files, 1);
        assert!(entry.children.contains_key("visible.txt"));
        assert!(!entry.children.contains_key(".secret"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn shows_hidden_files_when_requested() {
        let dir = test_dir("hidden-shown");
        fs::write(dir.join(".secret"), "x").unwrap();

        let mut scan_options = options();
        scan_options.show_hidden = true;
        let entry = scan_tree(&dir, &scan_options);

        assert!(entry.children.contains_key(".secret"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn filters_by_glob_and_files_only() {
        let dir = test_dir("glob-files");
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(src.join("lib.txt"), "hello\n").unwrap();

        let mut scan_options = options();
        scan_options.files_only = true;
        scan_options.glob = Some("src/*.rs");
        let entry = scan_tree(&dir, &scan_options);

        assert!(entry.children.contains_key("src"));
        let src_entry = entry.children.get("src").unwrap();
        assert!(src_entry.children.contains_key("main.rs"));
        assert!(!src_entry.children.contains_key("lib.txt"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn applies_limit_and_reports_truncation() {
        let dir = test_dir("limit");
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("b.txt"), "b").unwrap();
        fs::write(dir.join("c.txt"), "c").unwrap();

        let mut entry = scan_tree(&dir, &options());
        let truncated = apply_limit(&mut entry, 2);
        let counts = count_tree(&entry);

        assert!(truncated);
        assert_eq!(counts.files, 2);

        fs::remove_dir_all(dir).unwrap();
    }
}
