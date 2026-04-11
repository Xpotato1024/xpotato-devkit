# winget Final Submission Check Report (v0.1.6, 2026-04-11)

## Summary

- Confirmed GitHub Release `v0.1.6` is published and the standalone Windows installer asset exists.
- Refreshed the draft manifests under `packaging/winget/` so `PackageVersion`, installer URL, and installer SHA256 all match `v0.1.6`.
- Confirmed the current README and `winget` prep docs remain consistent with the published Windows asset naming and fixed metadata.
- Re-ran live-machine installer checks with the published `v0.1.6` installer and confirmed `--silent`, `--uninstall --silent`, `--unpack-only`, upgrade from installed `v0.1.5`, same-version reinstall, uninstall, and fresh reinstall all succeed with exit code `0`.
- Confirmed the PATH-managed branch as well: removing `%LOCALAPPDATA%\Xpotato\devkit` from User PATH before install caused the installer to record `path_added = true`, and uninstall removed the PATH entry again.

## Updated Files

- `packaging/winget/Xpotato.devkit.yaml`
- `packaging/winget/Xpotato.devkit.locale.en-US.yaml`
- `packaging/winget/Xpotato.devkit.installer.yaml`
- `docs/reports/winget_final_submission_check_v0.1.6_2026-04-11.md`

## Confirmed

- [x] `packaging/winget/Xpotato.devkit.installer.yaml` の `PackageVersion` を `0.1.6` に更新
- [x] `InstallerUrl` を `https://github.com/Xpotato1024/xpotato-devkit/releases/download/v0.1.6/devkit-installer-v0.1.6-x86_64-pc-windows-msvc.exe` に更新
- [x] `InstallerSha256` を `3a5d9f7ac7b5100e1cb7ca585df7dd0289220517b08a2477c5c3f9b37038ad78` に更新
- [x] `packaging/winget/Xpotato.devkit.yaml` と `packaging/winget/Xpotato.devkit.locale.en-US.yaml` の `PackageVersion` も `0.1.6` に更新
- [x] GitHub Release `v0.1.6` は公開済み
- [x] `devkit-installer-v0.1.6-x86_64-pc-windows-msvc.exe` が存在する
- [x] `devkit-v0.1.6-sha256.txt` が存在する
- [x] installer URL が `200 OK` を返す
- [x] README の Windows asset 記述は `devkit-installer-{tag}-x86_64-pc-windows-msvc.exe` を前提としており `v0.1.6` 実態と矛盾しない
- [x] `docs/install/windows-winget-prep.md` の固定方針は現 manifest の固定 metadata と一致する
- [x] `packaging/winget/README.md` の更新手順どおりに version / URL / SHA256 を更新した
- [x] `PackageIdentifier: Xpotato.devkit`
- [x] `PackageName: devkit`
- [x] `Publisher: Xpotato`
- [x] `Moniker: devkit`
- [x] `Commands: devkit`
- [x] `InstallerType: exe`
- [x] `Scope: user`
- [x] silent switch は `--silent`
- [x] 既定 install 先は docs 上 `%LOCALAPPDATA%\Xpotato\devkit`
- [x] uninstall 後は `install-manifest.json` に記録された管理下ファイルのみ削除と docs に明記されている
- [x] success exit code = `0`
- [x] operational failure exit code = `1`
- [x] argument / usage error exit code = `2`
- [x] manifest の version / URL / SHA256 は `0.1.6` に一致
- [x] standalone installer asset は実在
- [x] install 先が User PATH に無い状態では `install-manifest.json` に `path_added = true` が記録される
- [x] `path_added = true` の install 後、uninstall は User PATH から install 先を除去する
- [x] `devkit-installer.exe --silent` が成功する
- [x] `devkit-installer.exe --uninstall --silent` が成功する
- [x] `devkit-installer.exe --unpack-only` が成功する
- [x] silent install / uninstall が通る
- [x] 未インストール環境で install
- [x] install 後に `devkit --help`
- [x] install 後に `devkit search text ...` が動作
- [x] `0.1.5` から `0.1.6` へ再 install / upgrade が成功
- [x] uninstall が成功
- [x] reinstall が成功
- [x] docs / manifest / release asset の不整合がないことを live installer 実行結果込みで最終確認

## Verification

- Queried GitHub Release metadata for `v0.1.6` and confirmed `published_at = 2026-04-11T09:52:14Z`.
- Downloaded `devkit-v0.1.6-sha256.txt` and copied the installer SHA256 from the published checksum file.
- Checked the published installer URL and confirmed HTTP status `200`.
- Compared the updated manifests with:
  - `README.md`
  - `docs/install/windows-winget-prep.md`
  - `packaging/winget/README.md`
  - `docs/install/windows-installation.md`
- Live installer verification using `C:\Users\miyut\Downloads\devkit-installer-v0.1.6-x86_64-pc-windows-msvc.exe`:
  - `--unpack-only --install-dir %TEMP%\devkit-unpack-only-v0.1.6` returned exit code `0` and reported `PATH status: Skipped (--unpack-only)`.
  - The pre-existing default install under `%LOCALAPPDATA%\Xpotato\devkit` was on `v0.1.5`; running `--silent` upgraded it to `v0.1.6` and rewrote `install-manifest.json` to `version = v0.1.6` and `installer_version = v0.1.6`.
  - With a fresh-shell-style PATH composed from current machine and user PATH values, `where devkit` resolved `%LOCALAPPDATA%\Xpotato\devkit\devkit.exe` first, and both `devkit --help` and `devkit search text winget --limit 3 --brief` succeeded with exit code `0`.
  - A same-version `--silent` reinstall succeeded with exit code `0`.
  - `devkit-installer-v0.1.6-x86_64-pc-windows-msvc.exe --uninstall --silent` succeeded with exit code `0` and removed `%LOCALAPPDATA%\Xpotato\devkit`.
  - A fresh `--silent` install from the uninstalled state succeeded with exit code `0`.
  - The generated `%LOCALAPPDATA%\Xpotato\devkit\uninstall.exe --silent` also succeeded with exit code `0`.
  - A final `--silent` reinstall succeeded with exit code `0`; the machine was left with `%LOCALAPPDATA%\Xpotato\devkit\devkit.exe` on `v0.1.6`.
  - After temporarily removing `%LOCALAPPDATA%\Xpotato\devkit` from User PATH, a new `--silent` install returned exit code `0`, wrote `path_added = true` to `install-manifest.json`, and restored the install dir into User PATH.
  - From that `path_added = true` state, `--uninstall --silent` returned exit code `0`, removed `%LOCALAPPDATA%\Xpotato\devkit`, and removed the PATH entry.
  - A final `--silent` install returned exit code `0` and left the machine in `v0.1.6` installed state with `path_added = true`.

## Remaining Risk

- No repo-side checklist gap remains for the `v0.1.6` winget draft manifest and installer contract.
- The current interactive shell may still keep an older PATH snapshot until restarted; this is expected and consistent with the docs note that newly opened shells observe PATH changes more reliably.
