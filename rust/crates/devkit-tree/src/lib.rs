use ignore::{WalkBuilder, overrides::OverrideBuilder};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug)]
pub struct TreeEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub children: BTreeMap<String, TreeEntry>,
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

pub fn scan_tree(
    root: &Path,
    max_depth: Option<usize>,
    extensions: Option<&[String]>,
    dirs_only: bool,
    use_gitignore: bool,
    extra_ignore: &[String],
) -> TreeEntry {
    let mut builder = WalkBuilder::new(root);

    let mut ov = OverrideBuilder::new(root);
    for ig in extra_ignore {
        let _ = ov.add(&format!("!{}", ig));
    }
    if let Ok(o) = ov.build() {
        builder.overrides(o);
    }

    if let Some(depth) = max_depth {
        builder.max_depth(Some(depth));
    }

    if !use_gitignore {
        builder
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false);
    }
    builder.hidden(false);

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
        if dirs_only && !is_dir {
            continue;
        }

        if !is_dir && let Some(exts) = extensions {
            let ext = path
                .extension()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if !exts.iter().any(|e| e.trim_start_matches('.') == ext) {
                continue;
            }
        }

        if let Ok(rel_path) = path.strip_prefix(root) {
            let mut current = &mut root_entry;
            let components: Vec<_> = rel_path.iter().collect();
            for (i, c) in components.iter().enumerate() {
                let name = c.to_string_lossy().to_string();
                let c_is_dir = if i == components.len() - 1 {
                    is_dir
                } else {
                    true
                };
                let size = if c_is_dir {
                    0
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };

                current = current
                    .children
                    .entry(name.clone())
                    .or_insert_with(|| TreeEntry::new(name, c_is_dir, size));
            }
        }
    }

    if extensions.is_some() {
        prune_empty_dirs(&mut root_entry);
    }

    root_entry
}

fn prune_empty_dirs(entry: &mut TreeEntry) -> bool {
    let mut to_remove = Vec::new();
    for (k, v) in entry.children.iter_mut() {
        let has_content = prune_empty_dirs(v);
        if !has_content && v.is_dir {
            to_remove.push(k.clone());
        }
    }
    for k in to_remove {
        entry.children.remove(&k);
    }
    !entry.children.is_empty() || !entry.is_dir
}

pub fn format_tree(
    entry: &TreeEntry,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    lines: &mut Vec<String>,
) {
    let display = if is_root {
        format!("{}/", entry.name)
    } else {
        if entry.is_dir {
            format!("{}/", entry.name)
        } else {
            format!("{} ({})", entry.name, format_size(entry.size))
        }
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
        let mut sorted_children: Vec<_> = entry.children.values().collect();
        sorted_children.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        let len = sorted_children.len();
        for (i, child) in sorted_children.iter().enumerate() {
            format_tree(child, &child_prefix, i == len - 1, false, lines);
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

pub fn tree_summary(entry: &TreeEntry) -> String {
    let mut dirs = 0;
    let mut files = 0;
    fn count(e: &TreeEntry, d: &mut usize, f: &mut usize) {
        if e.is_dir {
            *d += 1;
            for c in e.children.values() {
                count(c, d, f);
            }
        } else {
            *f += 1;
        }
    }
    count(entry, &mut dirs, &mut files);
    let dirs = dirs.saturating_sub(1);
    format!("{} directories, {} files", dirs, files)
}
