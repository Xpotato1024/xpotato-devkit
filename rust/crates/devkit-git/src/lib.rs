use std::process::Command;

pub mod diff;
pub mod doc;
pub mod git;

pub fn run_git_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let error_msg = if !stderr.is_empty() { stderr } else { stdout };
        return Err(format!("Git command failed: {}", error_msg));
    }
    Ok(stdout)
}
