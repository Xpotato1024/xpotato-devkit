use std::error::Error;
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};

#[cfg(target_os = "windows")]
pub fn add_user_path_entry(entry: &Path) -> Result<bool, Box<dyn Error>> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env_key, _) = hkcu.create_subkey("Environment")?;
    let current: String = env_key.get_value("Path").unwrap_or_default();
    let normalized_entry = normalize_path_fragment(entry);
    let mut fragments = split_path_list(&current);

    if fragments
        .iter()
        .any(|fragment| normalize_path_fragment(fragment) == normalized_entry)
    {
        return Ok(false);
    }

    fragments.push(entry.to_string_lossy().to_string());
    let updated = fragments.join(";");
    env_key.set_value("Path", &updated)?;
    broadcast_environment_change()?;
    Ok(true)
}

#[cfg(not(target_os = "windows"))]
pub fn add_user_path_entry(_entry: &Path) -> Result<bool, Box<dyn Error>> {
    Err("PATH updates are only supported on Windows".into())
}

#[cfg(target_os = "windows")]
pub fn remove_user_path_entry(entry: &Path) -> Result<(), Box<dyn Error>> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = match hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE) {
        Ok(key) => key,
        Err(_) => return Ok(()),
    };
    let current: String = env_key.get_value("Path").unwrap_or_default();
    let normalized_entry = normalize_path_fragment(entry);
    let fragments: Vec<String> = split_path_list(&current)
        .into_iter()
        .filter(|fragment| normalize_path_fragment(fragment) != normalized_entry)
        .collect();
    env_key.set_value("Path", &fragments.join(";"))?;
    broadcast_environment_change()?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn remove_user_path_entry(_entry: &Path) -> Result<(), Box<dyn Error>> {
    Err("PATH updates are only supported on Windows".into())
}

pub fn split_path_list(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|fragment| !fragment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn normalize_path_fragment(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
fn broadcast_environment_change() -> Result<(), Box<dyn Error>> {
    let environment: Vec<u16> = OsStr::new("Environment")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut result = 0usize;
    let status = unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            environment.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result,
        )
    };

    if status == 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_path_fragments_and_ignores_empty_entries() {
        let fragments = split_path_list(r"C:\One; ;C:\Two;;");
        assert_eq!(
            fragments,
            vec![r"C:\One".to_string(), r"C:\Two".to_string()]
        );
    }

    #[test]
    fn normalizes_path_fragments_case_insensitively() {
        assert_eq!(
            normalize_path_fragment("C:\\Xpotato\\Devkit\\"),
            normalize_path_fragment("c:\\xpotato\\devkit")
        );
    }
}
