from __future__ import annotations

import argparse
import sys
from pathlib import Path


def require_contains(path: Path, needle: str, label: str, failures: list[str]) -> None:
    text = path.read_text(encoding="utf-8")
    if needle not in text:
        failures.append(f"{label}: missing `{needle}` in {path}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check that devkit release metadata is aligned with tag-based version injection."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[4],
        help="Repository root to inspect.",
    )
    args = parser.parse_args()

    root = args.repo_root.resolve()
    release_workflow = root / ".github" / "workflows" / "release.yml"
    cli_main = root / "rust" / "crates" / "devkit-cli" / "src" / "main.rs"
    installer_main = root / "rust" / "crates" / "devkit-installer" / "src" / "main.rs"

    failures: list[str] = []

    require_contains(
        release_workflow,
        "DEVKIT_RELEASE_VERSION: ${{ github.ref_name }}",
        "release workflow",
        failures,
    )
    require_contains(
        cli_main,
        'option_env!("DEVKIT_RELEASE_VERSION")',
        "devkit-cli version source",
        failures,
    )
    require_contains(
        cli_main,
        "#[command(author, version = RELEASE_VERSION, about, long_about = None)]",
        "devkit-cli clap version",
        failures,
    )
    require_contains(
        installer_main,
        'option_env!("DEVKIT_RELEASE_VERSION")',
        "installer version source",
        failures,
    )
    require_contains(
        installer_main,
        '#[command(author, version = RELEASE_VERSION, about = "Native Windows installer for devkit")]',
        "installer clap version",
        failures,
    )
    require_contains(
        installer_main,
        "version: RELEASE_VERSION.to_string()",
        "installer manifest version",
        failures,
    )
    require_contains(
        installer_main,
        "installer_version: RELEASE_VERSION.to_string()",
        "installer manifest installer_version",
        failures,
    )

    if failures:
        print("FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print("OK: release version alignment checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
