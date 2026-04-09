# Tag-Triggered Release

This repository publishes release binaries only from tag pushes that match `v*`.
Normal pushes to `main` and pull requests do not create releases.

## Trigger

- Create a SemVer-style tag such as `v0.1.0`
- Push the tag to the remote repository

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Build Matrix

The release workflow builds these targets:

- Linux: `x86_64-unknown-linux-gnu`
- Windows: `x86_64-pc-windows-msvc` plus the native `devkit-installer`
- macOS Apple Silicon: `aarch64-apple-darwin`

## Published assets

The workflow publishes:

- `devkit-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- `devkit-<tag>-x86_64-pc-windows-msvc.zip`
- `devkit-installer-<tag>-x86_64-pc-windows-msvc.exe`
- `devkit-<tag>-aarch64-apple-darwin.tar.gz`
- `devkit-<tag>-sha256.txt`

The Windows zip contains `devkit-installer.exe` only. The standalone `.exe` is published separately for direct download and `winget` consumption.

## Release Flow

1. Push a `v*` tag
2. GitHub Actions builds the target binaries
3. The workflow packages platform assets and the standalone Windows installer
4. The release job downloads every artifact
5. The release job generates `devkit-<tag>-sha256.txt`
6. GitHub Releases publishes all assets together

## Failure Handling

- If any build fails, the workflow fails and no release is published
- Fix the build problem, then push a new tag or recreate the failed tag according to maintainer policy
- Keep tag pushes intentional; do not use `main` pushes for release publication
