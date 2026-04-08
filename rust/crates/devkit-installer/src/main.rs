mod manifest;
mod windows_path;

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

#[derive(Parser, Debug)]
#[command(author, version, about = "Native Windows installer for devkit")]
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

    /// Add the install directory to the user PATH
    #[arg(long)]
    add_to_path: bool,

    /// Override the default install directory
    #[arg(long)]
    install_dir: Option<PathBuf>,
}

#[derive(Debug)]
struct InstallPaths {
    install_dir: PathBuf,
    devkit_exe: PathBuf,
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
    let package_dir = current_exe
        .parent()
        .ok_or("could not resolve the installer package directory")?;
    let payload_exe = package_dir.join("devkit.exe");

    if !payload_exe.is_file() {
        return Err("missing bundled payload: devkit.exe".into());
    }

    let paths = install_paths(cli.install_dir.clone().unwrap_or_else(default_install_dir));
    fs::create_dir_all(&paths.install_dir)?;

    copy_file(&payload_exe, &paths.devkit_exe)?;
    copy_file(current_exe, &paths.uninstall_exe)?;

    let mut path_added = false;
    if cli.add_to_path && !cli.unpack_only {
        match add_user_path_entry(&paths.install_dir) {
            Ok(added) => {
                path_added = added;
            }
            Err(err) => {
                eprintln!("Warning: PATH update failed: {err}");
            }
        }
    }

    let manifest = InstallManifest {
        product: "devkit".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        install_dir: paths.install_dir.to_string_lossy().to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        installed_files: vec![
            "devkit.exe".to_string(),
            "uninstall.exe".to_string(),
            "install-manifest.json".to_string(),
        ],
        path_added,
        path_value: if path_added {
            Some(paths.install_dir.to_string_lossy().to_string())
        } else {
            None
        },
        installer_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    write_manifest(&paths.manifest_path, &manifest)?;

    println!("Installed devkit to {}", paths.install_dir.display());
    println!("Binary: {}", paths.devkit_exe.display());
    println!("Uninstall: {}", paths.uninstall_exe.display());
    println!("PATH added: {}", if path_added { "Yes" } else { "No" });

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

    println!("Uninstall complete.");
    println!("Install directory: {}", paths.install_dir.display());
    Ok(())
}

fn install_paths(install_dir: PathBuf) -> InstallPaths {
    InstallPaths {
        devkit_exe: install_dir.join("devkit.exe"),
        uninstall_exe: install_dir.join("uninstall.exe"),
        manifest_path: install_dir.join("install-manifest.json"),
        install_dir,
    }
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

fn schedule_self_delete(path: &Path) -> Result<(), Box<dyn Error>> {
    let command = format!(
        "ping 127.0.0.1 -n 2 > NUL & del /F /Q \"{file}\" & rmdir \"{dir}\" 2>NUL",
        file = path.display(),
        dir = path.parent().unwrap_or_else(|| Path::new(".")).display()
    );
    Command::new("cmd").args(["/C", &command]).spawn()?;
    Ok(())
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
}
