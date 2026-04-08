use serde_derive::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct DevkitConfig {
    #[serde(default)]
    pub encoding: EncodingConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct GitConfig {
    #[serde(default)]
    pub lang: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct EncodingConfig {
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub path: String,
}

pub fn resolve_config_path(start: &Path) -> Option<PathBuf> {
    if let Ok(explicit) = env::var("DEVKIT_CONFIG") {
        let path = PathBuf::from(explicit);
        let resolved = if path.is_absolute() {
            path
        } else {
            start.join(path)
        };
        return Some(resolved);
    }

    let root = find_project_root(start);
    let candidate = root.join("devkit.toml");
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

pub fn config_base_dir(start: &Path) -> PathBuf {
    resolve_config_path(start)
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| start.to_path_buf())
}

pub fn find_project_root(start: &Path) -> PathBuf {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

    for _ in 0..32 {
        if current.join("devkit.toml").is_file() {
            return current;
        }
        if current.parent().is_none() {
            break;
        }
        current = current.parent().unwrap().to_path_buf();
    }

    start.to_path_buf()
}

pub fn load_config(cwd: &Path) -> Result<DevkitConfig, std::io::Error> {
    if let Some(config_path) = resolve_config_path(cwd) {
        let content = fs::read_to_string(&config_path)?;
        match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    } else {
        Ok(DevkitConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
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
            let path = std::env::temp_dir().join(format!("devkit-core-test-{}", unique));
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
    fn finds_project_root_by_devkit_toml() {
        let temp = TempDir::new();
        let root = temp.path.join("repo");
        let nested = root.join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("devkit.toml"), "[git]\nlang = \"en\"\n").unwrap();

        assert_eq!(
            find_project_root(&nested).canonicalize().unwrap(),
            root.canonicalize().unwrap()
        );
    }

    #[test]
    fn loads_metrics_config_from_parent_root() {
        let _guard = env_lock().lock().unwrap();
        let temp = TempDir::new();
        let root = temp.path.join("repo");
        let nested = root.join("rust");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            root.join("devkit.toml"),
            "[metrics]\nenabled = true\npath = \"metrics.jsonl\"\n",
        )
        .unwrap();

        let config = load_config(&nested).unwrap();
        assert!(config.metrics.enabled);
        assert_eq!(config.metrics.path, "metrics.jsonl");
    }

    #[test]
    fn load_config_prefers_devkit_config_env_var() {
        let _guard = env_lock().lock().unwrap();
        let temp = TempDir::new();
        let repo = temp.path.join("repo");
        let custom = temp.path.join("custom");
        fs::create_dir_all(repo.join("nested")).unwrap();
        fs::create_dir_all(&custom).unwrap();
        fs::write(repo.join("devkit.toml"), "[git]\nlang = \"ja\"\n").unwrap();
        let custom_config = custom.join("devkit.toml");
        fs::write(&custom_config, "[git]\nlang = \"en\"\n").unwrap();

        unsafe { env::set_var("DEVKIT_CONFIG", &custom_config) };
        let config = load_config(&repo.join("nested")).unwrap();
        unsafe { env::remove_var("DEVKIT_CONFIG") };

        assert_eq!(config.git.lang, "en");
    }
}
