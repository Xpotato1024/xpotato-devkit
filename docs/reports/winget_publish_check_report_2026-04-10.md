# winget Publish Check Report (2026-04-10)

## Summary

- Checked the published `v0.1.5` release assets, draft `winget` manifests, installer contract, and related docs.
- Updated the draft manifests under `packaging/winget/` to match the published `v0.1.5` release and pass `winget validate`.
- Found a real installer bug: reinstalling to the same directory rewrote `install-manifest.json` with `path_added=false`, which caused uninstall to leave the installer-added PATH entry behind.
- Fixed the reinstall PATH tracking bug on this branch and added unit and Windows E2E coverage.
- As a result, `v0.1.5` should not be the final `winget` submission target; the next submission should use a new release tag after this fix ships.

## Updated Files

- `packaging/winget/Xpotato.devkit.yaml`
- `packaging/winget/Xpotato.devkit.locale.en-US.yaml`
- `packaging/winget/Xpotato.devkit.installer.yaml`
- `packaging/winget/README.md`
- `docs/install/windows-winget-prep.md`
- `rust/crates/devkit-installer/src/main.rs`
- `rust/crates/devkit-installer/tests/e2e.rs`

## Verification

- Confirmed GitHub Release `v0.1.5` exists and includes `devkit-installer-v0.1.5-x86_64-pc-windows-msvc.exe`.
- Confirmed the installer SHA256 from `devkit-v0.1.5-sha256.txt`:
  - `9903d054caba87c430d7a7b9612c516bded252fefebd50f4f9da2f34440470bf`
- Confirmed the release installer URL returns `200 OK`.
- Confirmed a fresh-shell-style PATH resolves the installed `devkit.exe` and `devkit search text` works.
- Verified:
  - installer help exposes `--silent`, `--uninstall`, `--unpack-only`, `--install-dir`, and `--add-to-path`
  - usage errors exit with code `2`
  - operational failures exit with code `1`
  - silent install and silent uninstall succeed with exit code `0`
  - `--unpack-only` leaves `path_added=false`
  - regular install adds the target directory to the user PATH
- `winget validate` succeeded against a manifest-only temporary directory.
- `cargo test --manifest-path rust/Cargo.toml -p devkit-installer` succeeded, including the new reinstall/uninstall PATH regression coverage.

## Remaining Work

- Cut and publish a new release tag that includes the reinstall PATH tracking fix.
- Refresh `packaging/winget/` to that released version's `PackageVersion`, installer URL, and SHA256.
- Re-run `winget validate` and perform the final submission against the new released assets, not against `v0.1.5`.
