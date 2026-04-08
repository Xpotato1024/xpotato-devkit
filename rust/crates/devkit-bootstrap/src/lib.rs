use std::env;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("devkit-bootstrap-test-{}", unique));
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
}
