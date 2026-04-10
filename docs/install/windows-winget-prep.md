# Windows winget Preparation

この文書は、`devkit` を `winget` 公開できる状態へ寄せるための repo 内ソースをまとめたものです。実際の `winget-pkgs` 提出は次段階ですが、manifest 作成に必要な前提はこの repo 内で追えるようにします。

## 現在の方針

- PackageIdentifier: `Xpotato.devkit`
- PackageName: `devkit`
- Publisher: `Xpotato`
- Moniker: `devkit`
- Commands: `devkit`
- InstallerType: `exe`
- Scope: `user`
- 既定 install 先: `%LOCALAPPDATA%\Xpotato\devkit`
- silent switch: `--silent`

draft manifest は `packaging/winget/` を参照してください。

## Windows release assets

Windows release では次を公開します。

- `devkit-<tag>-x86_64-pc-windows-msvc.zip`
- `devkit-installer-<tag>-x86_64-pc-windows-msvc.exe`
- `devkit-<tag>-sha256.txt`

`winget` では standalone installer asset を使う前提です。

## Installer contract

インストーラーは CLI 実装です。標準的な利用形は次の通りです。

```powershell
.\devkit-installer.exe
.\devkit-installer.exe --silent
.\devkit-installer.exe --unpack-only
.\devkit-installer.exe --uninstall --silent
```

サポートする重要オプション:

- `--uninstall`
- `--unpack-only`
- `--install-dir <path>`
- `--add-to-path`
- `--silent`

## Exit code

- `0`: success
- `1`: operational failure
- `2`: argument / usage error

`--silent` は成功時の通常 stdout を抑制します。error 出力は stderr に残ります。

## install / upgrade / uninstall の扱い

- 未インストール時: 既定で install + PATH 追加
- 再 install 時: 同じ install 先へ上書き配置
- upgrade 時: 同じ installer を新バージョンで実行する前提
- `--unpack-only`: PATH 追加を行わず payload の配置だけ実施
- uninstall 後: `install-manifest.json` に記録された管理下ファイルのみ削除

## SHA256 の取得

release workflow は公開 asset 一式に対して `devkit-<tag>-sha256.txt` を生成します。`winget` manifest 作成時はそのファイルの SHA256 を使ってください。

## version の扱い

`PackageVersion` は `winget` manifest だけの都合で先行して増やさず、提出対象の公開済み release と一致させます。
検証中に installer や uninstall の不具合が見つかって修正が必要になった場合は、その修正版を新しい release tag として公開してから manifest を更新してください。

## 提出前チェック

- standalone installer asset 名が docs と manifest draft に一致している
- `devkit-installer.exe --silent` と `--uninstall --silent` が動作する
- user-scope install と install 先が docs と一致している
- checksum file が release asset に含まれている
