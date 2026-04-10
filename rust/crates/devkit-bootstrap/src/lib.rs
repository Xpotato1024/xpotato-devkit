use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_repo_root(start: &Path) -> Result<PathBuf, String> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

    for _ in 0..32 {
        if current.join("devkit.toml").is_file()
            && current
                .join("rust")
                .join("crates")
                .join("devkit-cli")
                .join("Cargo.toml")
                .is_file()
        {
            return Ok(current);
        }
        if current.parent().is_none() {
            break;
        }
        current = current.parent().unwrap().to_path_buf();
    }

    Err("Could not find the devkit repository root.".to_string())
}

pub fn cargo_bin_dir() -> PathBuf {
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        return PathBuf::from(cargo_home).join("bin");
    }

    if let Ok(user_profile) = env::var("USERPROFILE") {
        return PathBuf::from(user_profile).join(".cargo").join("bin");
    }

    PathBuf::from(".cargo").join("bin")
}

pub fn bootstrap_self(repo_root: &Path) -> Result<PathBuf, String> {
    let rust_root = repo_root.join("rust");
    let status = Command::new("cargo")
        .args(["install", "--path", "crates/devkit-cli", "--force"])
        .current_dir(&rust_root)
        .status()
        .map_err(|error| format!("Failed to execute cargo: {}", error))?;

    if !status.success() {
        return Err("cargo install failed.".to_string());
    }

    Ok(cargo_bin_dir())
}

pub fn sync_repo_skills(
    repo_root: &Path,
    target_root: &Path,
    force: bool,
) -> Result<Vec<String>, String> {
    let source_root = repo_root.join("SKILLs");
    if !source_root.is_dir() {
        return Err(format!(
            "Could not find repo-bundled skills at {}.",
            source_root.display()
        ));
    }

    let target_skills_root = target_root.join("SKILLs");
    fs::create_dir_all(&target_skills_root).map_err(|error| {
        format!(
            "Failed to create {}: {}",
            target_skills_root.display(),
            error
        )
    })?;

    let mut copied = Vec::new();
    for entry in fs::read_dir(&source_root)
        .map_err(|error| format!("Failed to read {}: {}", source_root.display(), error))?
    {
        let entry = entry
            .map_err(|error| format!("Failed to read {}: {}", source_root.display(), error))?;
        let source_path = entry.path();
        if !source_path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let target_path = target_skills_root.join(&name);
        if target_path.exists() {
            if !force {
                return Err(format!(
                    "{} already exists. Re-run with --force to replace it.",
                    target_path.display()
                ));
            }

            if target_path.is_dir() {
                fs::remove_dir_all(&target_path).map_err(|error| {
                    format!("Failed to replace {}: {}", target_path.display(), error)
                })?;
            } else {
                fs::remove_file(&target_path).map_err(|error| {
                    format!("Failed to replace {}: {}", target_path.display(), error)
                })?;
            }
        }

        copy_tree(&source_path, &target_path)?;
        copied.push(name);
    }

    copied.sort();
    Ok(copied)
}

pub fn init_agents_template(
    agents_path: &Path,
    skills_dir_label: &str,
    skills_dir_path: &Path,
    force: bool,
) -> Result<Vec<String>, String> {
    if agents_path.exists() && !force {
        return Err(format!(
            "{} already exists. Use --force to overwrite it.",
            agents_path.display()
        ));
    }

    let bundled_skills = list_skill_names(skills_dir_path)?;
    let content = render_agents_template(skills_dir_label, &bundled_skills);

    if let Some(parent) = agents_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
    }

    fs::write(agents_path, content)
        .map_err(|error| format!("Failed to write {}: {}", agents_path.display(), error))?;
    Ok(bundled_skills)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("Failed to create {}: {}", destination.display(), error))?;

    for entry in fs::read_dir(source)
        .map_err(|error| format!("Failed to read {}: {}", source.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("Failed to read {}: {}", source.display(), error))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
            }
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!(
                    "Failed to copy {} to {}: {}",
                    source_path.display(),
                    destination_path.display(),
                    error
                )
            })?;
        }
    }

    Ok(())
}

fn list_skill_names(skills_dir: &Path) -> Result<Vec<String>, String> {
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(skills_dir)
        .map_err(|error| format!("Failed to read {}: {}", skills_dir.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("Failed to read {}: {}", skills_dir.display(), error))?;
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    names.sort();
    Ok(names)
}

fn render_agents_template(skills_dir_label: &str, bundled_skills: &[String]) -> String {
    let mut template = format!(
        "# AGENTS.md\n\n## Scope\n- This repository may use repo-bundled Codex skills stored under `{skills_dir_label}/`.\n- Keep repository-specific workflow in those skills instead of relying on global assumptions.\n\n## Preferred Skills\n- If a task matches a repo-bundled skill, invoke it before improvising.\n- Prefer `--brief` or JSON-capable output when another tool or agent will consume command output.\n\n## Skill Usage\n- Read only the relevant `SKILL.md` for the skill you are about to use.\n- Follow linked references or scripts only as needed for the current task.\n- Do not bulk-load every skill when only one workflow is in scope.\n"
    );

    if bundled_skills.is_empty() {
        template.push_str(
            "\n## Bundled Skills\n- Add repo-bundled skills under the configured skills directory and list any repository-preferred ones here.\n",
        );
    } else {
        template.push_str("\n## Bundled Skills\n");
        for skill in bundled_skills {
            template.push_str(&format!("- `{skill}`\n"));
        }
    }

    template
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
                "devkit-bootstrap-test-{}-{}-{}",
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
    fn finds_repo_root_using_rust_cli_marker() {
        let temp = TempDir::new();
        let root = temp.path.join("repo");
        let nested = root.join("a").join("b");
        fs::create_dir_all(root.join("rust").join("crates").join("devkit-cli")).unwrap();
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("devkit.toml"), "[git]\nlang = \"ja\"\n").unwrap();
        fs::write(
            root.join("rust")
                .join("crates")
                .join("devkit-cli")
                .join("Cargo.toml"),
            "[package]\nname = \"devkit-cli\"\n",
        )
        .unwrap();

        assert_eq!(
            find_repo_root(&nested).unwrap().canonicalize().unwrap(),
            root.canonicalize().unwrap()
        );
    }

    #[test]
    fn sync_repo_skills_copies_skill_directories() {
        let temp = TempDir::new();
        let repo_root = temp.path.join("repo");
        let target_root = temp.path.join("workspace");
        fs::create_dir_all(repo_root.join("SKILLs").join("alpha")).unwrap();
        fs::write(
            repo_root.join("SKILLs").join("alpha").join("SKILL.md"),
            "# alpha\n",
        )
        .unwrap();
        fs::write(
            repo_root.join("SKILLs").join("alpha").join("notes.txt"),
            "hello\n",
        )
        .unwrap();

        let copied = sync_repo_skills(&repo_root, &target_root, false).unwrap();

        assert_eq!(copied, vec!["alpha".to_string()]);
        assert!(
            target_root
                .join("SKILLs")
                .join("alpha")
                .join("SKILL.md")
                .is_file()
        );
        assert!(
            target_root
                .join("SKILLs")
                .join("alpha")
                .join("notes.txt")
                .is_file()
        );
    }

    #[test]
    fn sync_repo_skills_requires_force_when_target_exists() {
        let temp = TempDir::new();
        let repo_root = temp.path.join("repo");
        let target_root = temp.path.join("workspace");
        fs::create_dir_all(repo_root.join("SKILLs").join("alpha")).unwrap();
        fs::write(
            repo_root.join("SKILLs").join("alpha").join("SKILL.md"),
            "# alpha\n",
        )
        .unwrap();
        fs::create_dir_all(target_root.join("SKILLs").join("alpha")).unwrap();

        let error = sync_repo_skills(&repo_root, &target_root, false).unwrap_err();
        assert!(error.contains("--force"));
    }

    #[test]
    fn init_agents_template_lists_detected_skills() {
        let temp = TempDir::new();
        let agents_path = temp.path.join("workspace").join("AGENTS.md");
        let skills_dir = temp.path.join("workspace").join("SKILLs");
        fs::create_dir_all(skills_dir.join("alpha")).unwrap();
        fs::create_dir_all(skills_dir.join("beta")).unwrap();
        fs::write(skills_dir.join("alpha").join("SKILL.md"), "# alpha\n").unwrap();
        fs::write(skills_dir.join("beta").join("SKILL.md"), "# beta\n").unwrap();

        let names = init_agents_template(&agents_path, "SKILLs", &skills_dir, false).unwrap();
        let content = fs::read_to_string(&agents_path).unwrap();

        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
        assert!(content.contains("`alpha`"));
        assert!(content.contains("`beta`"));
        assert!(content.contains("`SKILLs/`"));
    }

    #[test]
    fn init_agents_template_refuses_overwrite_without_force() {
        let temp = TempDir::new();
        let agents_path = temp.path.join("workspace").join("AGENTS.md");
        if let Some(parent) = agents_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&agents_path, "existing\n").unwrap();

        let error = init_agents_template(
            &agents_path,
            "SKILLs",
            &temp.path.join("workspace").join("SKILLs"),
            false,
        )
        .unwrap_err();
        assert!(error.contains("--force"));
    }
}
