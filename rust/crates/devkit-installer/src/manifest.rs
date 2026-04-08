use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallManifest {
    pub product: String,
    pub version: String,
    pub install_dir: String,
    pub installed_at: String,
    pub installed_files: Vec<String>,
    pub path_added: bool,
    pub path_value: Option<String>,
    pub installer_version: String,
}

pub fn read_manifest(path: &Path) -> Result<InstallManifest, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn write_manifest(path: &Path, manifest: &InstallManifest) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(manifest)?;
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, json)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp_path, path)?;
    Ok(())
}

pub fn resolve_install_dir(
    manifest: &InstallManifest,
    manifest_path: &Path,
    fallback: &Path,
) -> PathBuf {
    let manifest_dir = manifest_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| fallback.to_path_buf());
    let recorded_dir = PathBuf::from(&manifest.install_dir);

    if recorded_dir.as_os_str().is_empty() {
        manifest_dir
    } else {
        recorded_dir
    }
}

pub fn manifest_path_value(manifest: &InstallManifest) -> Option<PathBuf> {
    if !manifest.path_added {
        return None;
    }

    manifest.path_value.as_ref().and_then(|value| {
        let path = PathBuf::from(value);
        if path.as_os_str().is_empty() {
            None
        } else {
            Some(path)
        }
    })
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
            "devkit-installer-{name}-{}-{unique}-{}",
            std::process::id(),
            next_temp_id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn sample_manifest() -> InstallManifest {
        InstallManifest {
            product: "devkit".to_string(),
            version: "0.1.0".to_string(),
            install_dir: r"C:\Users\Example\AppData\Local\Xpotato\devkit".to_string(),
            installed_at: "2026-04-08T12:00:00+09:00".to_string(),
            installed_files: vec![
                "devkit.exe".to_string(),
                "uninstall.exe".to_string(),
                "install-manifest.json".to_string(),
            ],
            path_added: true,
            path_value: Some(r"C:\Users\Example\AppData\Local\Xpotato\devkit".to_string()),
            installer_version: "0.1.0".to_string(),
        }
    }

    #[test]
    fn manifest_round_trip_is_stable() {
        let manifest = sample_manifest();

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: InstallManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.product, "devkit");
        assert!(parsed.path_added);
        assert_eq!(parsed.installed_files.len(), 3);
    }

    #[test]
    fn resolve_install_dir_prefers_manifest_value() {
        let manifest = InstallManifest {
            product: "devkit".to_string(),
            version: "0.1.0".to_string(),
            install_dir: r"C:\Custom\Devkit".to_string(),
            installed_at: "2026-04-08T12:00:00+09:00".to_string(),
            installed_files: vec![],
            path_added: false,
            path_value: None,
            installer_version: "0.1.0".to_string(),
        };

        assert_eq!(
            resolve_install_dir(
                &manifest,
                Path::new(r"C:\Users\Example\AppData\Local\Xpotato\devkit\install-manifest.json"),
                Path::new(r"C:\Fallback\Devkit")
            ),
            PathBuf::from(r"C:\Custom\Devkit")
        );
    }

    #[test]
    fn manifest_path_value_requires_recorded_path_addition() {
        let manifest = InstallManifest {
            product: "devkit".to_string(),
            version: "0.1.0".to_string(),
            install_dir: r"C:\Custom\Devkit".to_string(),
            installed_at: "2026-04-08T12:00:00+09:00".to_string(),
            installed_files: vec![],
            path_added: true,
            path_value: Some(r"C:\Custom\Devkit".to_string()),
            installer_version: "0.1.0".to_string(),
        };

        assert_eq!(
            manifest_path_value(&manifest),
            Some(PathBuf::from(r"C:\Custom\Devkit"))
        );
    }

    #[test]
    fn write_manifest_replaces_existing_file_and_is_readable() {
        let dir = test_dir("manifest-write");
        let manifest_path = dir.join("install-manifest.json");
        fs::write(&manifest_path, "{invalid json").unwrap();

        let manifest = sample_manifest();
        write_manifest(&manifest_path, &manifest).unwrap();
        let parsed = read_manifest(&manifest_path).unwrap();

        assert_eq!(parsed.product, "devkit");
        assert_eq!(parsed.installed_files.len(), 3);
        assert!(!manifest_path.with_extension("json.tmp").exists());

        fs::remove_dir_all(dir).unwrap();
    }
}
