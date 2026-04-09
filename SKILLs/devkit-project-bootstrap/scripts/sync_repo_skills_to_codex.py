from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path


def copy_tree(source: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    for child in source.iterdir():
        target = destination / child.name
        if child.is_dir():
            if target.exists():
                shutil.rmtree(target)
            shutil.copytree(child, target)
        else:
            shutil.copy2(child, target)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Sync repo-bundled SKILLs into the local Codex skill store."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[4],
        help="Repository root containing SKILLs/.",
    )
    parser.add_argument(
        "--codex-skills",
        type=Path,
        default=Path.home() / ".codex" / "skills",
        help="Local Codex skills directory.",
    )
    args = parser.parse_args()

    repo_root = args.repo_root.resolve()
    codex_skills = args.codex_skills.resolve()
    source_root = repo_root / "SKILLs"

    if not source_root.is_dir():
        print(f"FAIL: missing repo skill directory: {source_root}")
        return 1

    codex_skills.mkdir(parents=True, exist_ok=True)

    copied: list[str] = []
    for skill_dir in sorted(source_root.iterdir()):
        if not skill_dir.is_dir():
            continue
        skill_md = skill_dir / "SKILL.md"
        if not skill_md.is_file():
            continue
        copy_tree(skill_dir, codex_skills / skill_dir.name)
        copied.append(skill_dir.name)

    print(f"OK: synced {len(copied)} skills to {codex_skills}")
    for name in copied:
        print(f"- {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
