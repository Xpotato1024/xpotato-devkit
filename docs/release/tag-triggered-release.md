# Tag-Triggered Release

This repository publishes release binaries only from tag pushes that match `v*`.
Normal pushes to `main` and pull requests do not create releases.

## Trigger

- Create a SemVer-style tag such as `v0.1.0`.
- Push the tag to the remote repository.

Example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Build Matrix

The release workflow builds the `devkit-cli` package for these targets:

- Linux: `x86_64-unknown-linux-gnu`
- Windows: `x86_64-pc-windows-msvc` plus the native `devkit-installer` bundle
- macOS Apple Silicon: `aarch64-apple-darwin`

The workflow packages the platform binary into an archive before upload.
On Windows, the archive also includes `devkit-installer.exe` so users can run the native installer after extraction.
Linux and macOS archives include the `devkit` binary only.

## Asset Names

Release assets follow this pattern:

```text
devkit-{tag}-{target}{ext}
```

Examples:

- `devkit-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `devkit-v0.1.0-x86_64-pc-windows-msvc.zip`
- `devkit-v0.1.0-aarch64-apple-darwin.tar.gz`

Archive formats:

- Windows: `.zip`
- Linux and macOS: `.tar.gz`

## Release Flow

1. Push a `v*` tag.
2. GitHub Actions builds the three release binaries.
3. The workflow uploads the packaged assets to a single GitHub Release.
4. The release is published without changelog automation.

## Failure Handling

- If any build fails, the workflow fails and no release is published.
- Fix the build problem, then push a new tag or delete and recreate the failed tag if that is the preferred release policy.
- Keep tag pushes intentional; do not use `main` pushes for release publication.
