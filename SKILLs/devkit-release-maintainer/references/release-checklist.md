# Release Checklist

Use this reference when actively preparing or validating a release.

## Pre-tag checks

1. Read the relevant release docs and recent reports.
2. Verify local fallback behavior:
   - `cargo test -p devkit-cli -p devkit-installer`
   - `cargo run -p devkit-cli -- -V`
3. Verify injected release behavior:
   - `DEVKIT_RELEASE_VERSION=<tag> cargo run -p devkit-cli -- -V`
   - `DEVKIT_RELEASE_VERSION=<tag> cargo run -p devkit-installer --bin devkit-installer -- --version`
4. Run the static checker when release-version wiring changed:
   - `uv run python scripts/check_release_version_alignment.py`

## Windows packaging checks

1. Build `devkit-cli`
2. Build `devkit-cleanup-helper`
3. Build `devkit-installer` with:
   - `DEVKIT_INSTALLER_PAYLOAD`
   - `DEVKIT_CLEANUP_HELPER`
   - `DEVKIT_RELEASE_VERSION`

## User-visible version contract

These should agree with the release tag:

- `devkit -V`
- `devkit-installer --version`
- installer manifest `version`
- installer manifest `installer_version`
