# Windows Installation

Windows releases ship a native installer and a zip archive that contains the same installer.

## Recommended asset

Prefer the standalone installer asset from the [Releases page](https://github.com/Xpotato1024/xpotato-devkit/releases):

- `devkit-installer-<tag>-x86_64-pc-windows-msvc.exe`

The zip asset remains available for users who prefer an archive download:

- `devkit-<tag>-x86_64-pc-windows-msvc.zip`

## Interactive install

Run the installer directly:

```powershell
.\devkit-installer.exe
```

By default the installer:

- installs to `%LOCALAPPDATA%\Xpotato\devkit`
- adds that directory to the user PATH
- writes `devkit.exe`
- writes `uninstall.exe`
- writes `install-manifest.json`

Use `--unpack-only` to skip PATH changes:

```powershell
.\devkit-installer.exe --unpack-only
```

## Silent install

For unattended execution, use `--silent`:

```powershell
.\devkit-installer.exe --silent
.\devkit-installer.exe --silent --install-dir C:\Tools\devkit
```

`--silent` suppresses normal stdout output on success. Error output remains on stderr.

## Uninstall

Run the generated uninstaller from the install directory:

```powershell
& "$env:LOCALAPPDATA\Xpotato\devkit\uninstall.exe"
```

For unattended uninstall:

```powershell
& "$env:LOCALAPPDATA\Xpotato\devkit\uninstall.exe" --silent
```

The uninstaller removes only the files recorded in `install-manifest.json`.

## Exit codes

- `0`: success
- `1`: operational failure
- `2`: argument / usage error

## Notes

- The installer carries its own embedded `devkit.exe` payload and can be redistributed as a single file.
- After PATH changes, the installer broadcasts a Windows environment update so newly opened shells can observe the updated PATH more reliably.
- Existing shells keep their current PATH. Open a new shell after install before checking `devkit`.
- If another `devkit.exe` is already earlier on PATH, the installer prints a warning in non-silent mode.
- This repository does not use a PowerShell-based installer.
- `winget` preparation notes and draft manifests live under [windows-winget-prep.md](windows-winget-prep.md) and `packaging/winget/`.
