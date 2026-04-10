#![cfg(target_os = "windows")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

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
        "devkit-installer-e2e-{name}-{}-{unique}-{}",
        std::process::id(),
        next_temp_id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn workspace_rust_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn build_devkit_cli() -> PathBuf {
    let workspace = workspace_rust_dir();
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .arg("build")
        .arg("-p")
        .arg("devkit-cli")
        .current_dir(&workspace)
        .status()
        .unwrap();
    assert!(status.success(), "failed to build devkit-cli for E2E test");

    workspace.join("target").join("debug").join("devkit.exe")
}

fn copy_file(source: &Path, destination: &Path) {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::copy(source, destination).unwrap();
}

fn wait_for_path_removal(path: &Path) {
    for _ in 0..100 {
        if !path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("path still exists after waiting: {}", path.display());
}

fn normalize_path_fragment(path: &Path) -> String {
    path.to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_ascii_lowercase()
}

fn user_path_fragments() -> Vec<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu.open_subkey("Environment").unwrap();
    let current: String = env_key.get_value("Path").unwrap_or_default();
    current
        .split(';')
        .map(str::trim)
        .filter(|fragment| !fragment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn user_path_contains(path: &Path) -> bool {
    let normalized = normalize_path_fragment(path);
    user_path_fragments()
        .iter()
        .any(|fragment| normalize_path_fragment(Path::new(fragment)) == normalized)
}

fn wait_for_user_path_presence(path: &Path) {
    for _ in 0..100 {
        if user_path_contains(path) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("user PATH does not contain {}", path.display());
}

fn wait_for_user_path_absence(path: &Path) {
    for _ in 0..100 {
        if !user_path_contains(path) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("user PATH still contains {}", path.display());
}

fn wait_for_files_removal(paths: &[&Path]) {
    for _ in 0..50 {
        if paths.iter().all(|path| !path.exists()) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let remaining: Vec<String> = paths
        .iter()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .collect();
    panic!("paths still exist after waiting: {}", remaining.join(", "));
}

#[test]
fn installer_round_trip_with_sidecar_payload() {
    let temp = test_dir("round-trip");
    let package_dir = temp.join("package");
    let install_dir = temp.join("install");
    fs::create_dir_all(&package_dir).unwrap();

    let installer_source = PathBuf::from(env!("CARGO_BIN_EXE_devkit-installer"));
    let helper_source = PathBuf::from(env!("CARGO_BIN_EXE_devkit-cleanup-helper"));
    let devkit_source = build_devkit_cli();
    let installer_path = package_dir.join("devkit-installer.exe");
    let helper_path = package_dir.join("devkit-cleanup-helper.exe");
    let sidecar_path = package_dir.join("devkit.exe");
    copy_file(&installer_source, &installer_path);
    copy_file(&helper_source, &helper_path);
    copy_file(&devkit_source, &sidecar_path);

    let install = Command::new(&installer_path)
        .arg("--silent")
        .arg("--unpack-only")
        .arg("--install-dir")
        .arg(&install_dir)
        .output()
        .unwrap();
    assert!(
        install.status.success(),
        "install failed: stdout={}; stderr={}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );
    assert!(String::from_utf8_lossy(&install.stdout).trim().is_empty());

    let manifest_path = install_dir.join("install-manifest.json");
    let installed_devkit = install_dir.join("devkit.exe");
    let uninstall_path = install_dir.join("uninstall.exe");
    assert!(installed_devkit.is_file());
    assert!(uninstall_path.is_file());
    assert!(manifest_path.is_file());
    assert!(install_dir.join("devkit-cleanup-helper.exe").is_file());

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["product"], "devkit");
    assert_eq!(manifest["path_added"], false);
    assert!(manifest["path_value"].is_null());

    let uninstall = Command::new(&uninstall_path)
        .arg("--silent")
        .arg("--install-dir")
        .arg(&install_dir)
        .output()
        .unwrap();
    assert!(
        uninstall.status.success(),
        "uninstall failed: stdout={}; stderr={}",
        String::from_utf8_lossy(&uninstall.stdout),
        String::from_utf8_lossy(&uninstall.stderr)
    );
    assert!(String::from_utf8_lossy(&uninstall.stdout).trim().is_empty());

    wait_for_files_removal(&[&installed_devkit, &manifest_path]);
    wait_for_path_removal(&install_dir);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn reinstall_preserves_path_tracking_for_uninstall() {
    let temp = test_dir("reinstall-path-tracking");
    let package_dir = temp.join("package");
    let install_dir = temp.join("install");
    fs::create_dir_all(&package_dir).unwrap();

    let installer_source = PathBuf::from(env!("CARGO_BIN_EXE_devkit-installer"));
    let helper_source = PathBuf::from(env!("CARGO_BIN_EXE_devkit-cleanup-helper"));
    let devkit_source = build_devkit_cli();
    let installer_path = package_dir.join("devkit-installer.exe");
    let helper_path = package_dir.join("devkit-cleanup-helper.exe");
    let sidecar_path = package_dir.join("devkit.exe");
    copy_file(&installer_source, &installer_path);
    copy_file(&helper_source, &helper_path);
    copy_file(&devkit_source, &sidecar_path);

    let install = Command::new(&installer_path)
        .arg("--silent")
        .arg("--install-dir")
        .arg(&install_dir)
        .output()
        .unwrap();
    assert!(
        install.status.success(),
        "install failed: stdout={}; stderr={}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );
    wait_for_user_path_presence(&install_dir);

    let manifest_path = install_dir.join("install-manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["path_added"], true);
    assert_eq!(manifest["path_value"], install_dir.to_string_lossy().to_string());

    let reinstall = Command::new(&installer_path)
        .arg("--silent")
        .arg("--install-dir")
        .arg(&install_dir)
        .output()
        .unwrap();
    assert!(
        reinstall.status.success(),
        "reinstall failed: stdout={}; stderr={}",
        String::from_utf8_lossy(&reinstall.stdout),
        String::from_utf8_lossy(&reinstall.stderr)
    );

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["path_added"], true);
    assert_eq!(manifest["path_value"], install_dir.to_string_lossy().to_string());

    let uninstall_path = install_dir.join("uninstall.exe");
    let uninstall = Command::new(&uninstall_path)
        .arg("--silent")
        .arg("--install-dir")
        .arg(&install_dir)
        .output()
        .unwrap();
    assert!(
        uninstall.status.success(),
        "uninstall failed: stdout={}; stderr={}",
        String::from_utf8_lossy(&uninstall.stdout),
        String::from_utf8_lossy(&uninstall.stderr)
    );

    wait_for_files_removal(&[&install_dir.join("devkit.exe"), &manifest_path]);
    wait_for_path_removal(&install_dir);
    wait_for_user_path_absence(&install_dir);
    fs::remove_dir_all(temp).unwrap();
}
