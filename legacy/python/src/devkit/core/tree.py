"""Core logic for compact project tree display.

Scans a directory tree while respecting .gitignore patterns and
devkit.toml ignore lists, producing a minimal text representation
suitable for AI consumption.
"""

from __future__ import annotations

import fnmatch
import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional, Set


# ---------------------------------------------------------------------------
# Gitignore parser (simple, no external deps)
# ---------------------------------------------------------------------------

def _parse_gitignore(gitignore_path: Path) -> List[str]:
    """Parse a .gitignore file into a list of patterns."""
    if not gitignore_path.is_file():
        return []
    patterns: List[str] = []
    for line in gitignore_path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        patterns.append(line)
    return patterns


def _matches_any(path: Path, root: Path, patterns: List[str]) -> bool:
    """Check if *path* matches any gitignore-style pattern."""
    rel = path.relative_to(root)
    rel_str = str(rel).replace("\\", "/")
    name = path.name

    for pat in patterns:
        pat_clean = pat.rstrip("/")
        # Directory-only pattern
        if pat.endswith("/"):
            if path.is_dir() and (fnmatch.fnmatch(name, pat_clean) or fnmatch.fnmatch(rel_str, pat_clean)):
                return True
            continue
        # Match by name or relative path
        if fnmatch.fnmatch(name, pat_clean):
            return True
        if fnmatch.fnmatch(rel_str, pat_clean):
            return True
        # Match with ** prefix
        if pat_clean.startswith("**/"):
            if fnmatch.fnmatch(rel_str, pat_clean[3:]):
                return True
        # Pattern in any subdirectory
        for part in rel.parts:
            if fnmatch.fnmatch(part, pat_clean):
                return True
    return False


# ---------------------------------------------------------------------------
# Tree data
# ---------------------------------------------------------------------------

@dataclass
class TreeEntry:
    name: str
    is_dir: bool
    size: int = 0
    children: List["TreeEntry"] = field(default_factory=list)


def _format_size(size: int) -> str:
    if size < 1024:
        return f"{size}B"
    if size < 1024 * 1024:
        return f"{size / 1024:.1f}KB"
    return f"{size / (1024 * 1024):.1f}MB"


# ---------------------------------------------------------------------------
# Scanner
# ---------------------------------------------------------------------------

def scan_tree(
    root: Path,
    *,
    max_depth: Optional[int] = None,
    extensions: Optional[Set[str]] = None,
    dirs_only: bool = False,
    use_gitignore: bool = True,
    extra_ignore: Optional[List[str]] = None,
) -> TreeEntry:
    """Scan a directory tree and return a `TreeEntry` hierarchy.

    Parameters
    ----------
    root : Path
        Root directory to scan.
    max_depth : int | None
        Maximum depth to descend.  ``None`` = unlimited.
    extensions : set[str] | None
        If given, only include files whose suffix (e.g. ``".py"``) is in this set.
    dirs_only : bool
        If True, include only directories.
    use_gitignore : bool
        If True, read and apply ``.gitignore`` at the root.
    extra_ignore : list[str] | None
        Additional ignore patterns (e.g. from devkit.toml).
    """
    ignore_patterns: List[str] = list(extra_ignore or [])
    if use_gitignore:
        ignore_patterns.extend(_parse_gitignore(root / ".gitignore"))

    def _scan(path: Path, depth: int) -> Optional[TreeEntry]:
        if _matches_any(path, root, ignore_patterns):
            return None

        if path.is_file():
            if dirs_only:
                return None
            if extensions and path.suffix.lower() not in extensions:
                return None
            return TreeEntry(name=path.name, is_dir=False, size=path.stat().st_size)

        if path.is_dir():
            if max_depth is not None and depth > max_depth:
                return TreeEntry(name=path.name, is_dir=True)

            children: List[TreeEntry] = []
            try:
                entries = sorted(path.iterdir(), key=lambda p: (not p.is_dir(), p.name.lower()))
            except PermissionError:
                return TreeEntry(name=path.name, is_dir=True)

            for child in entries:
                entry = _scan(child, depth + 1)
                if entry:
                    children.append(entry)

            # Skip empty dirs when filtering by extension
            if extensions and not children and path != root:
                return None

            return TreeEntry(name=path.name, is_dir=True, children=children)

        return None

    result = _scan(root, 0)
    return result or TreeEntry(name=root.name, is_dir=True)


def format_tree(entry: TreeEntry, prefix: str = "", is_last: bool = True, is_root: bool = True) -> List[str]:
    """Format a `TreeEntry` into indented text lines."""
    lines: List[str] = []

    if is_root:
        display = entry.name + "/"
    else:
        display = entry.name
        if entry.is_dir:
            display += "/"
        else:
            display += f" ({_format_size(entry.size)})"

    if is_root:
        lines.append(display)
    else:
        connector = "└── " if is_last else "├── "
        lines.append(prefix + connector + display)

    if entry.is_dir:
        child_prefix = prefix + ("    " if is_last or is_root else "│   ")
        for i, child in enumerate(entry.children):
            is_child_last = i == len(entry.children) - 1
            lines.extend(format_tree(child, child_prefix, is_child_last, is_root=False))

    return lines


def tree_summary(entry: TreeEntry) -> str:
    """Return a summary line like '12 directories, 28 files'."""
    dirs = 0
    files = 0

    def _count(e: TreeEntry) -> None:
        nonlocal dirs, files
        if e.is_dir:
            dirs += 1
            for c in e.children:
                _count(c)
        else:
            files += 1

    _count(entry)
    # Don't count the root
    dirs = max(0, dirs - 1)
    return f"{dirs} directories, {files} files"
