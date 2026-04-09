# Windows Installation

Windows releases ship a native installer bundle.
The archive contains:

- `devkit-installer.exe`

## Recommended path

1. Open the [Releases page](https://github.com/Xpotato1024/xpotato-devkit/releases).
2. Download the Windows archive for the tag you want.
3. Extract the archive.
4. Run `devkit-installer.exe`.

The installer adds `%LOCALAPPDATA%\Xpotato\devkit` to the user PATH by default.
If you only want to unpack the files without changing PATH, pass `--unpack-only`:

```powershell
.\devkit-installer.exe --unpack-only
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
- `--add-to-path` remains accepted for compatibility, but PATH addition is now the default behavior.
- `--unpack-only` is the explicit opt-out for PATH updates.
- After PATH changes, the installer broadcasts a Windows environment update so newly opened shells can observe the updated PATH more reliably.
- Existing shells keep their current PATH. Open a new shell after install before checking `devkit`.
- If another `devkit.exe` is already earlier on PATH, the installer prints a warning so users can verify which binary will be resolved.
- The release archive no longer needs a sidecar `devkit.exe`; the installer carries the payload itself and can be redistributed as a single file after extraction.
- The generated `uninstall.exe` resolves `install-manifest.json` from its own install directory by default.
- This repository does not use a PowerShell-based installer.
