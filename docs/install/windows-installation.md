# Windows Installation

Windows releases ship a native installer bundle.
The bundle contains:

- `devkit.exe`
- `devkit-installer.exe`

## Recommended path

1. Open the [Releases page](https://github.com/Xpotato1024/xpotato-devkit/releases).
2. Download the Windows archive for the tag you want.
3. Extract the archive.
4. Run `devkit-installer.exe`.

If you want `devkit` on the user PATH, pass `--add-to-path`:

```powershell
.\devkit-installer.exe --add-to-path
```

The installer copies the CLI into the current user profile and writes:

- `devkit.exe`
- `uninstall.exe`
- `install-manifest.json`

The default install location is:

```text
%LOCALAPPDATA%\Xpotato\devkit
```

## Uninstall

Run the generated `uninstall.exe` from the install directory:

```powershell
& "$env:LOCALAPPDATA\Xpotato\devkit\uninstall.exe"
```

The uninstaller removes only the files recorded in `install-manifest.json`.

## Notes

- The Windows release asset remains the `x86_64-pc-windows-msvc` zip archive.
- PATH is optional and does not gate a successful install.
- After PATH changes, the installer broadcasts a Windows environment update so newly opened shells can observe the updated PATH more reliably.
- The generated `uninstall.exe` resolves `install-manifest.json` from its own install directory by default.
- This repository does not use a PowerShell-based installer.
