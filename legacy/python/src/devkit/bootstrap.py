from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def find_repo_root(start: Path | None = None) -> Path:
    current = (start or Path.cwd()).resolve()
    for _ in range(32):
        if (current / "pyproject.toml").is_file() and (current / "src" / "devkit").is_dir():
            return current
        if current.parent == current:
            break
        current = current.parent
    raise FileNotFoundError("Could not find the devkit repository root.")


def run_command(args: list[str], cwd: Path, capture_output: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=str(cwd),
        check=True,
        text=True,
        encoding="utf-8",
        capture_output=capture_output,
    )


def get_tool_bin(repo_root: Path, home: Path | None = None) -> Path:
    try:
        result = run_command(["uv", "tool", "dir", "--bin"], cwd=repo_root, capture_output=True)
        path = result.stdout.strip()
        if path:
            return Path(path)
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass
    return (home or Path.home()) / ".local" / "bin"


def bootstrap_self(repo_root: Path) -> Path:
    run_command(["uv", "tool", "install", "--force", "--editable", "."], cwd=repo_root)
    run_command(["uv", "tool", "update-shell"], cwd=repo_root)
    return get_tool_bin(repo_root)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Install devkit from the current checkout as a user tool.")
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=None,
        help="Path to the devkit repository root. Defaults to auto-detection from the current directory.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    repo_root = args.repo_root or find_repo_root()
    tool_bin = bootstrap_self(repo_root)
    print(
        "Bootstrap complete. "
        f"If the current shell does not see devkit yet, restart it or add {tool_bin} to PATH."
    )
    return 0

