mod manifest;
mod windows_path;
include!(concat!(env!("OUT_DIR"), "/embedded_payload.rs"));

use clap::Parser;
use manifest::{
    InstallManifest, manifest_path_value, read_manifest, resolve_install_dir, write_manifest,
};
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use windows_path::{add_user_path_entry, remove_user_path_entry};

const RELEASE_VERSION: &str = match option_env!("DEVKIT_RELEASE_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(author, version = RELEASE_VERSION, about = "Native Windows installer for devkit")]
struct Cli {
    /// Install the bundled devkit payload into the user profile
    #[arg(long)]
    install: bool,

    /// Uninstall devkit from the user profile
    #[arg(long)]
    uninstall: bool,

    /// Copy the payload without adding the install directory to PATH
    #[arg(long)]
    unpack_only: bool,

    /// Retained for compatibility; PATH addition is the default unless --unpack-only is used
    #[arg(long)]
    add_to_path: bool,

    /// Override the default install directory
    #[arg(long)]
    install_dir: Option<PathBuf>,

    /// Suppress regular stdout output on success
    #[arg(long)]
    silent: bool,
}

#[derive(Debug)]
struct InstallPaths {
    install_dir: PathBuf,
    devkit_exe: PathBuf,
    cleanup_helper_exe: PathBuf,
    uninstall_exe: PathBuf,
    manifest_path: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let current_exe = std::env::current_exe()?;
    let mode = resolve_mode(&cli, &current_exe);

    match mode {
        Mode::Install => install(&cli, &current_exe),
        Mode::Uninstall => uninstall(&cli, &current_exe),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Install,
    Uninstall,
}

fn resolve_mode(cli: &Cli, current_exe: &Path) -> Mode {
    if cli.uninstall {
        return Mode::Uninstall;
    }

    let file_name = current_exe
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if file_name == "uninstall.exe" {
        Mode::Uninstall
    } else {
        Mode::Install
    }
}

fn install(cli: &Cli, current_exe: &Path) -> Result<(), Box<dyn Error>> {
    let paths = install_paths(cli.install_dir.clone().unwrap_or_else(default_install_dir));
    fs::create_dir_all(&paths.install_dir)?;

    let payload_source = install_payload(current_exe, &paths.devkit_exe)?;
    copy_file(current_exe, &paths.uninstall_exe)?;
    let cleanup_helper_installed =
        install_cleanup_helper_sidecar(current_exe, &paths.cleanup_helper_exe)?;

    if cli.add_to_path && cli.unpack_only {
        eprintln!("Warning: --add-to-path is ignored when --unpack-only is set.");
    }

    let path_status = if should_add_to_path(cli) {
        match add_user_path_entry(&paths.install_dir) {
            Ok(true) => PathStatus::Added,
            Ok(false) => PathStatus::AlreadyPresent,
            Err(err) => {
                eprintln!("Warning: PATH update failed: {err}");
                PathStatus::Failed
            }
        }
    } else {
        PathStatus::Skipped
    };

    let mut installed_files = vec![
        "devkit.exe".to_string(),
        "uninstall.exe".to_string(),
        "install-manifest.json".to_string(),
    ];
    if cleanup_helper_installed {
        installed_files.push("devkit-cleanup-helper.exe".to_string());
    }

    let (path_added, path_value) =
        install_manifest_path_record(&paths.manifest_path, &paths.install_dir, path_status);

    let manifest = InstallManifest {
        product: "devkit".to_string(),
        version: RELEASE_VERSION.to_string(),
        install_dir: paths.install_dir.to_string_lossy().to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        installed_files,
        path_added,
        path_value,
        installer_version: RELEASE_VERSION.to_string(),
    };

    write_manifest(&paths.manifest_path, &manifest)?;

    emit_stdout(
        cli,
        &format!("Installed devkit to {}", paths.install_dir.display()),
    );
    emit_stdout(cli, &format!("Binary: {}", paths.devkit_exe.display()));
    emit_stdout(
        cli,
        &format!("Uninstall: {}", paths.uninstall_exe.display()),
    );
    emit_stdout(cli, &format!("Payload source: {}", payload_source.label()));
    emit_stdout(cli, &format!("PATH status: {}", path_status.label()));

    if should_add_to_path(cli) {
        emit_stdout(cli, "Open a new shell to use the updated PATH.");
    }

    if let Some(message) = install_resolution_message(cli, &paths.devkit_exe) {
        emit_stdout(cli, &format!("Resolution: {message}"));
    }

    for message in install_warnings(cli, &paths.devkit_exe) {
        emit_stdout(cli, &format!("Warning: {message}"));
    }

    Ok(())
}

fn uninstall(cli: &Cli, current_exe: &Path) -> Result<(), Box<dyn Error>> {
    let manifest_path = resolve_manifest_path(cli, current_exe);
    let manifest: InstallManifest = read_manifest(&manifest_path)?;
    let install_dir = resolve_install_dir(&manifest, &manifest_path, &default_install_dir());
    let paths = install_paths(install_dir.clone());
    let current_exe_canon = current_exe
        .canonicalize()
        .unwrap_or_else(|_| current_exe.to_path_buf());

    if let Some(path_value) = manifest_path_value(&manifest)
        && let Err(err) = remove_user_path_entry(&path_value)
    {
        eprintln!("Warning: PATH removal failed: {err}");
    }

    let uninstall_name = current_exe
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    for relative in &manifest.installed_files {
        let target = paths.install_dir.join(relative);
        if !target.exists() {
            continue;
        }

        let target_is_current =
            uninstall_name == "uninstall.exe" && same_path(&current_exe_canon, &target);

        if target_is_current {
            schedule_self_delete(&target)?;
        } else {
            fs::remove_file(&target)?;
        }
    }

    remove_dir_if_empty(&paths.install_dir)?;

    emit_stdout(cli, "Uninstall complete.");
    emit_stdout(
        cli,
        &format!("Install directory: {}", paths.install_dir.display()),
    );
    Ok(())
}

fn install_paths(install_dir: PathBuf) -> InstallPaths {
    InstallPaths {
        devkit_exe: install_dir.join("devkit.exe"),
        cleanup_helper_exe: install_dir.join("devkit-cleanup-helper.exe"),
        uninstall_exe: install_dir.join("uninstall.exe"),
        manifest_path: install_dir.join("install-manifest.json"),
        install_dir,
    }
}

fn install_manifest_path_record(
    manifest_path: &Path,
    install_dir: &Path,
    path_status: PathStatus,
) -> (bool, Option<String>) {
    match path_status {
        PathStatus::Added => (true, Some(install_dir.to_string_lossy().to_string())),
        PathStatus::AlreadyPresent => {
            let preserved = read_manifest(manifest_path)
                .ok()
                .and_then(|manifest| manifest_path_value(&manifest))
                .filter(|path| paths_equivalent(path, install_dir))
                .map(|path| path.to_string_lossy().to_string());

            if let Some(path_value) = preserved {
                (true, Some(path_value))
            } else {
                (false, None)
            }
        }
        PathStatus::Skipped | PathStatus::Failed => (false, None),
    }
}

fn paths_equivalent(left: &Path, right: &Path) -> bool {
    fn normalize(path: &Path) -> String {
        path.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_ascii_lowercase()
    }

    normalize(left) == normalize(right)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathStatus {
    Added,
    AlreadyPresent,
    Skipped,
    Failed,
}

impl PathStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::AlreadyPresent => "Already present",
            Self::Skipped => "Skipped (--unpack-only)",
            Self::Failed => "Failed",
        }
    }
}

fn should_add_to_path(cli: &Cli) -> bool {
    !cli.unpack_only
}

fn emit_stdout(cli: &Cli, message: &str) {
    if !cli.silent {
        println!("{message}");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PayloadSource {
    Embedded,
    Sidecar,
}

impl PayloadSource {
    fn label(self) -> &'static str {
        match self {
            Self::Embedded => "Embedded",
            Self::Sidecar => "Sidecar devkit.exe",
        }
    }
}

fn install_payload(
    current_exe: &Path,
    destination: &Path,
) -> Result<PayloadSource, Box<dyn Error>> {
    if let Some(payload) = EMBEDDED_PAYLOAD {
        write_file(destination, payload)?;
        return Ok(PayloadSource::Embedded);
    }

    let package_dir = current_exe
        .parent()
        .ok_or("could not resolve the installer package directory")?;
    let payload_exe = package_dir.join("devkit.exe");

    if !payload_exe.is_file() {
        return Err("missing embedded payload and sidecar devkit.exe".into());
    }

    copy_file(&payload_exe, destination)?;
    Ok(PayloadSource::Sidecar)
}

fn default_install_dir() -> PathBuf {
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public\AppData\Local"));
    local_app_data.join("Xpotato").join("devkit")
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(source, destination)?;
    Ok(())
}

fn write_file(destination: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(destination, bytes)?;
    Ok(())
}

fn schedule_self_delete(path: &Path) -> Result<(), Box<dyn Error>> {
    let helper_path = std::env::temp_dir().join(format!(
        "devkit-uninstall-cleanup-{}-{}.exe",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    install_cleanup_helper(&helper_path)?;
    Command::new(&helper_path)
        .arg("--target")
        .arg(path)
        .arg("--dir")
        .arg(path.parent().unwrap_or_else(|| Path::new(".")))
        .arg("--self-path")
        .arg(&helper_path)
        .spawn()?;
    Ok(())
}

fn install_cleanup_helper(destination: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(helper) = EMBEDDED_CLEANUP_HELPER {
        write_file(destination, helper)?;
        return Ok(());
    }

    let current_exe = std::env::current_exe()?;
    let package_dir = current_exe
        .parent()
        .ok_or("could not resolve the cleanup helper package directory")?;
    let helper_exe = package_dir.join("devkit-cleanup-helper.exe");

    if !helper_exe.is_file() {
        return Err("missing embedded cleanup helper and sidecar devkit-cleanup-helper.exe".into());
    }

    copy_file(&helper_exe, destination)?;
    Ok(())
}

fn install_cleanup_helper_sidecar(
    current_exe: &Path,
    destination: &Path,
) -> Result<bool, Box<dyn Error>> {
    if EMBEDDED_CLEANUP_HELPER.is_some() {
        return Ok(false);
    }

    let package_dir = current_exe
        .parent()
        .ok_or("could not resolve the cleanup helper package directory")?;
    let helper_exe = package_dir.join("devkit-cleanup-helper.exe");
    if !helper_exe.is_file() {
        return Err("missing sidecar devkit-cleanup-helper.exe".into());
    }

    copy_file(&helper_exe, destination)?;
    Ok(true)
}

fn same_path(left: &Path, right: &Path) -> bool {
    let left = left
        .canonicalize()
        .unwrap_or_else(|_| left.to_path_buf())
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    let right = right
        .canonicalize()
        .unwrap_or_else(|_| right.to_path_buf())
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    left == right
}

fn find_all_commands_on_path(command_name: &str, path_value: &OsStr) -> Vec<PathBuf> {
    std::env::split_paths(path_value)
        .map(|dir| dir.join(command_name))
        .filter(|candidate| candidate.is_file())
        .collect()
}

fn install_resolution_message(cli: &Cli, installed_path: &Path) -> Option<String> {
    if !should_add_to_path(cli) {
        return Some(
            "PATH update skipped; use the installed binary directly or add it manually".to_string(),
        );
    }

    let hits = std::env::var_os("PATH")
        .map(|value| find_all_commands_on_path("devkit.exe", &value))
        .unwrap_or_default();

    if hits.is_empty() {
        return Some(format!(
            "new shells should resolve {}",
            installed_path.display()
        ));
    }

    let first = &hits[0];
    if same_path(first, installed_path) {
        Some(format!(
            "current shell already resolves {}",
            first.display()
        ))
    } else {
        Some(format!(
            "current shell resolves {}; new shells should resolve {}",
            first.display(),
            installed_path.display()
        ))
    }
}

fn install_warnings(cli: &Cli, installed_path: &Path) -> Vec<String> {
    let mut warnings = Vec::new();

    if !should_add_to_path(cli) {
        return warnings;
    }

    let Some(path_value) = std::env::var_os("PATH") else {
        return warnings;
    };
    let hits = find_all_commands_on_path("devkit.exe", &path_value);
    if hits.is_empty() {
        return warnings;
    }

    let installed_on_current_path = hits
        .iter()
        .any(|candidate| same_path(candidate, installed_path));
    let first_other = hits
        .iter()
        .find(|candidate| !same_path(candidate, installed_path))
        .cloned();

    if let Some(existing) = first_other {
        if installed_on_current_path {
            if !same_path(&hits[0], installed_path) {
                warnings.push(format!(
                    "another devkit.exe is ahead of the installed binary on PATH: {}",
                    existing.display()
                ));
                warnings.push(
                    "a new shell may continue to resolve that binary until PATH order is changed"
                        .to_string(),
                );
            }
        } else {
            warnings.push(format!(
                "the current shell still resolves another devkit.exe from PATH: {}",
                existing.display()
            ));
            warnings.push(
                "open a new shell, then verify the resolved binary with `where devkit`".to_string(),
            );
        }
    }

    warnings
}

fn resolve_manifest_path(cli: &Cli, current_exe: &Path) -> PathBuf {
    if let Some(install_dir) = &cli.install_dir {
        return install_dir.join("install-manifest.json");
    }

    if current_exe
        .file_name()
        .and_then(OsStr::to_str)
        .map(|name| name.eq_ignore_ascii_case("uninstall.exe"))
        .unwrap_or(false)
        && let Some(parent) = current_exe.parent()
    {
        return parent.join("install-manifest.json");
    }

    default_install_dir().join("install-manifest.json")
}

fn remove_dir_if_empty(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        return Ok(());
    }

    if fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }

    Ok(())
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

    #[test]
    fn resolve_manifest_path_uses_uninstall_parent_by_default() {
        let cli = Cli {
            install: false,
            uninstall: false,
            unpack_only: false,
            add_to_path: false,
            install_dir: None,
            silent: false,
        };
        let uninstall_path = std::env::temp_dir()
            .join("Xpotato")
            .join("devkit")
            .join("uninstall.exe");

        let path = resolve_manifest_path(&cli, &uninstall_path);

        assert_eq!(
            path,
            uninstall_path
                .parent()
                .unwrap()
                .join("install-manifest.json")
        );
    }

    #[test]
    fn installer_adds_path_by_default() {
        let cli = Cli {
            install: false,
            uninstall: false,
            unpack_only: false,
            add_to_path: false,
            install_dir: None,
            silent: false,
        };

        assert!(should_add_to_path(&cli));
    }

    #[test]
    fn unpack_only_skips_path_update() {
        let cli = Cli {
            install: false,
            uninstall: false,
            unpack_only: true,
            add_to_path: true,
            install_dir: None,
            silent: false,
        };

        assert!(!should_add_to_path(&cli));
    }

    #[test]
    fn finds_first_matching_command_from_path_value() {
        let dir = test_dir("path-search");
        let bin_dir = dir.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let candidate = bin_dir.join("devkit.exe");
        fs::write(&candidate, b"binary").unwrap();
        let path_value = std::env::join_paths([bin_dir.as_path()]).unwrap();

        let found = find_all_commands_on_path("devkit.exe", &path_value)
            .into_iter()
            .next()
            .unwrap();

        assert_eq!(found, candidate);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn finds_all_matching_commands_from_path_value() {
        let dir = test_dir("path-search-all");
        let bin_one = dir.join("bin-one");
        let bin_two = dir.join("bin-two");
        fs::create_dir_all(&bin_one).unwrap();
        fs::create_dir_all(&bin_two).unwrap();
        let one = bin_one.join("devkit.exe");
        let two = bin_two.join("devkit.exe");
        fs::write(&one, b"one").unwrap();
        fs::write(&two, b"two").unwrap();
        let path_value = std::env::join_paths([bin_one.as_path(), bin_two.as_path()]).unwrap();

        let found = find_all_commands_on_path("devkit.exe", &path_value);

        assert_eq!(found, vec![one, two]);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn install_warnings_report_stale_current_shell_path() {
        let dir = test_dir("path-warning-stale");
        let other = dir.join("other").join("devkit.exe");
        let installed = dir.join("installed").join("devkit.exe");
        fs::create_dir_all(other.parent().unwrap()).unwrap();
        fs::write(&other, b"other").unwrap();
        fs::create_dir_all(installed.parent().unwrap()).unwrap();
        fs::write(&installed, b"installed").unwrap();
        let original = std::env::var_os("PATH");
        let path_value = std::env::join_paths([other.parent().unwrap()]).unwrap();
        unsafe {
            std::env::set_var("PATH", &path_value);
        }

        let cli = Cli {
            install: false,
            uninstall: false,
            unpack_only: false,
            add_to_path: false,
            install_dir: None,
            silent: false,
        };
        let warnings = install_warnings(&cli, &installed);

        assert!(
            warnings
                .iter()
                .any(|item| item.contains("current shell still resolves"))
        );

        match original {
            Some(value) => unsafe { std::env::set_var("PATH", value) },
            None => unsafe { std::env::remove_var("PATH") },
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn resolution_message_reports_current_and_next_shell_paths() {
        let dir = test_dir("path-resolution");
        let other_dir = dir.join("other");
        let installed_dir = dir.join("installed");
        fs::create_dir_all(&other_dir).unwrap();
        fs::create_dir_all(&installed_dir).unwrap();
        let other = other_dir.join("devkit.exe");
        let installed = installed_dir.join("devkit.exe");
        fs::write(&other, b"other").unwrap();
        fs::write(&installed, b"installed").unwrap();
        let original = std::env::var_os("PATH");
        let path_value =
            std::env::join_paths([other_dir.as_path(), installed_dir.as_path()]).unwrap();
        unsafe {
            std::env::set_var("PATH", &path_value);
        }

        let cli = Cli {
            install: false,
            uninstall: false,
            unpack_only: false,
            add_to_path: false,
            install_dir: None,
            silent: false,
        };
        let message = install_resolution_message(&cli, &installed).unwrap();

        assert!(message.contains("current shell resolves"));
        assert!(message.contains(&installed.display().to_string()));

        match original {
            Some(value) => unsafe { std::env::set_var("PATH", value) },
            None => unsafe { std::env::remove_var("PATH") },
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn remove_dir_if_empty_deletes_empty_directory() {
        let dir = test_dir("remove-empty");
        remove_dir_if_empty(&dir).unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn remove_dir_if_empty_keeps_non_empty_directory() {
        let dir = test_dir("keep-non-empty");
        fs::write(dir.join("devkit.exe"), b"binary").unwrap();

        remove_dir_if_empty(&dir).unwrap();

        assert!(dir.exists());
        assert!(dir.join("devkit.exe").exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn install_manifest_path_record_preserves_existing_added_path() {
        let dir = test_dir("manifest-path-record-preserve");
        let install_dir = dir.join("install");
        fs::create_dir_all(&install_dir).unwrap();
        let manifest_path = install_dir.join("install-manifest.json");

        let manifest = InstallManifest {
            product: "devkit".to_string(),
            version: "v0.1.5".to_string(),
            install_dir: install_dir.to_string_lossy().to_string(),
            installed_at: "2026-04-10T00:00:00Z".to_string(),
            installed_files: vec!["devkit.exe".to_string()],
            path_added: true,
            path_value: Some(install_dir.to_string_lossy().to_string()),
            installer_version: "v0.1.5".to_string(),
        };
        write_manifest(&manifest_path, &manifest).unwrap();

        let (path_added, path_value) =
            install_manifest_path_record(&manifest_path, &install_dir, PathStatus::AlreadyPresent);

        assert!(path_added);
        assert_eq!(path_value, Some(install_dir.to_string_lossy().to_string()));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn install_manifest_path_record_does_not_claim_manual_path_entries() {
        let dir = test_dir("manifest-path-record-manual");
        let install_dir = dir.join("install");
        fs::create_dir_all(&install_dir).unwrap();
        let manifest_path = install_dir.join("install-manifest.json");

        let manifest = InstallManifest {
            product: "devkit".to_string(),
            version: "v0.1.5".to_string(),
            install_dir: install_dir.to_string_lossy().to_string(),
            installed_at: "2026-04-10T00:00:00Z".to_string(),
            installed_files: vec!["devkit.exe".to_string()],
            path_added: false,
            path_value: None,
            installer_version: "v0.1.5".to_string(),
        };
        write_manifest(&manifest_path, &manifest).unwrap();

        let (path_added, path_value) =
            install_manifest_path_record(&manifest_path, &install_dir, PathStatus::AlreadyPresent);

        assert!(!path_added);
        assert_eq!(path_value, None);

        fs::remove_dir_all(dir).unwrap();
    }
}
