# winget Draft Metadata

このディレクトリは `winget-pkgs` 提出用の下書き置き場です。ここにある YAML は repo 内の source-of-truth として管理し、実提出時に version と URL と SHA256 を更新します。

## 更新手順

1. `PackageVersion` を対象 release tag に合わせる
2. `InstallerUrl` を `devkit-installer-<tag>-x86_64-pc-windows-msvc.exe` に合わせる
3. `InstallerSha256` を `devkit-<tag>-sha256.txt` から転記する
4. release notes / docs と `PackageIdentifier` などのメタデータ整合性を確認する

`PackageVersion` は manifest を更新した日付や review の都合ではなく、`winget-pkgs` に提出する公開済み release と一致させます。
manifest だけ先に直した場合でも、対応する release asset が未公開ならその version では提出しません。
公開済み release の検証後に機能修正が入った場合は、同じ version のまま提出せず、次の release tag に合わせて URL と SHA256 を更新し直します。

## 固定方針

- PackageIdentifier: `Xpotato.devkit`
- PackageName: `devkit`
- Publisher: `Xpotato`
- Moniker: `devkit`
- Command: `devkit`
- InstallerType: `exe`
- Scope: `user`
- Silent switch: `--silent`
