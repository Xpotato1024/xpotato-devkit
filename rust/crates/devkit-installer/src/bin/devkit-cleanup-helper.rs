use clap::Parser;
use std::error::Error;
use std::fs;
use std::io::Write;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Storage::FileSystem::{MOVEFILE_DELAY_UNTIL_REBOOT, MoveFileExW};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Internal cleanup helper for devkit installer"
)]
struct Cli {
    #[arg(long)]
    target: PathBuf,

    #[arg(long)]
    dir: PathBuf,

    #[arg(long)]
    self_path: Option<PathBuf>,
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
    let self_path = cli.self_path.as_deref().unwrap_or(&current_exe);

    let target_removed = wait_for_removal_target(&cli.target);
    let dir_removed = try_remove_dir(&cli.dir);

    if !target_removed {
        let scheduled = schedule_delete_on_reboot(&cli.target);
        write_cleanup_log(&format!(
            "target cleanup fallback: path={}, scheduled_on_reboot={scheduled}",
            cli.target.display()
        ));
    }

    if !dir_removed {
        let scheduled = schedule_delete_on_reboot(&cli.dir);
        write_cleanup_log(&format!(
            "directory cleanup fallback: path={}, scheduled_on_reboot={scheduled}",
            cli.dir.display()
        ));
    }

    schedule_helper_self_delete(self_path)?;
    Ok(())
}

fn wait_for_removal_target(target: &Path) -> bool {
    for _ in 0..60 {
        if fs::remove_file(target).is_ok() || !target.exists() {
            return true;
        }
        thread::sleep(Duration::from_secs(1));
    }
    !target.exists()
}

fn try_remove_dir(path: &Path) -> bool {
    for _ in 0..60 {
        if fs::remove_dir(path).is_ok() || !path.exists() {
            return true;
        }
        thread::sleep(Duration::from_secs(1));
    }
    !path.exists()
}

fn schedule_helper_self_delete(path: &Path) -> Result<(), Box<dyn Error>> {
    let command = format!(
        "ping 127.0.0.1 -n 2 > NUL & del /F /Q \"{file}\"",
        file = path.display()
    );
    Command::new("cmd").args(["/C", &command]).spawn()?;
    Ok(())
}

fn schedule_delete_on_reboot(path: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe { MoveFileExW(wide.as_ptr(), std::ptr::null(), MOVEFILE_DELAY_UNTIL_REBOOT) != 0 }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        false
    }
}

fn write_cleanup_log(message: &str) {
    let log_path = std::env::temp_dir().join("devkit-cleanup-helper.log");
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = writeln!(file, "{message}");
    }
}
