"""Core logic for Markdown section manipulation.

Provides safe, frontmatter-aware operations for appending to, replacing,
ensuring, and augmenting sections in Markdown documents.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import List, Optional, Tuple


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------

_FRONTMATTER_RE = re.compile(r"\A---\r?\n", re.MULTILINE)


def _split_frontmatter(text: str) -> Tuple[str, str]:
    """Split *text* into (frontmatter_block, body).

    If the file starts with ``---``, the frontmatter includes everything up to
    and including the closing ``---`` line.  Otherwise *frontmatter_block* is
    an empty string.
    """
    if not text.startswith("---"):
        return "", text

    # Find the closing --- (skip the opening one)
    end = text.find("\n---", 3)
    if end == -1:
        return "", text

    # Include the closing --- line and its newline
    close_pos = text.index("---", end + 1)
    after_close = close_pos + 3
    # consume trailing newline(s) of closing ---
    while after_close < len(text) and text[after_close] in ("\r", "\n"):
        after_close += 1

    return text[:after_close], text[after_close:]


def _heading_level(line: str) -> int:
    stripped = line.strip()
    if not stripped.startswith("#"):
        return 0
    parts = stripped.split()
    if not parts:
        return 0
    marker = parts[0]
    if set(marker) != {"#"}:
        return 0
    return len(marker)


def _heading_text(line: str) -> str:
    stripped = line.strip()
    level = _heading_level(stripped)
    if level == 0:
        return ""
    return stripped[level:].strip()


def _find_section(
    lines: List[str], heading: str, *, exact: bool = True
) -> Tuple[int, int, int]:
    """Find a section by heading in *lines*.

    Returns ``(start_idx, content_start_idx, end_idx)`` where:
    - *start_idx* is the heading line itself (inclusive).
    - *content_start_idx* is the first line after the heading (for append).
    - *end_idx* is the exclusive end of the section body.

    Raises ``ValueError`` when the heading is not found.
    """
    matches: List[Tuple[int, int]] = []  # (line_idx, level)
    for i, line in enumerate(lines):
        level = _heading_level(line)
        if level == 0:
            continue
        text = _heading_text(line)
        full = line.strip()

        if exact:
            if text == heading or full == heading:
                matches.append((i, level))
        else:
            if heading in full:
                matches.append((i, level))

    if not matches:
        raise ValueError(f"Heading '{heading}' not found.")
    if len(matches) > 1:
        locs = ", ".join(f"L{m[0]+1}" for m in matches)
        raise ValueError(
            f"Heading '{heading}' matched {len(matches)} sections ({locs}). "
            "Provide a more specific heading."
        )

    start_idx, level = matches[0]
    content_start = start_idx + 1

    # Find end of section
    end_idx = len(lines)
    for i in range(start_idx + 1, len(lines)):
        cur_level = _heading_level(lines[i])
        if cur_level > 0 and cur_level <= level:
            end_idx = i
            break

    return start_idx, content_start, end_idx


def _write_back(filepath: Path, frontmatter: str, lines: List[str]) -> None:
    filepath.write_text(frontmatter + "\n".join(lines), encoding="utf-8")


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def append_to_section(
    filepath: Path,
    heading: str,
    content: str,
    *,
    dry_run: bool = False,
) -> str:
    """Append *content* to the end of the section identified by *heading*.

    Returns the updated full text.
    """
    text = filepath.read_text(encoding="utf-8")
    frontmatter, body = _split_frontmatter(text)
    lines = body.splitlines(keepends=True)

    _, _, end_idx = _find_section(
        [l.rstrip("\n").rstrip("\r") for l in lines], heading
    )

    # Ensure content ends with newline
    if content and not content.endswith("\n"):
        content += "\n"

    # Insert before end_idx
    new_lines = lines[:end_idx] + [content] + lines[end_idx:]
    result = frontmatter + "".join(new_lines)

    if not dry_run:
        filepath.write_text(result, encoding="utf-8")
    return result


def replace_section(
    filepath: Path,
    heading: str,
    content: str,
    *,
    keep_heading: bool = True,
    dry_run: bool = False,
) -> str:
    """Replace the body of the section identified by *heading* with *content*.

    If *keep_heading* is True (default), the heading line itself is preserved.
    Returns the updated full text.
    """
    text = filepath.read_text(encoding="utf-8")
    frontmatter, body = _split_frontmatter(text)
    lines = body.splitlines(keepends=True)
    plain_lines = [l.rstrip("\n").rstrip("\r") for l in lines]

    start_idx, content_start, end_idx = _find_section(plain_lines, heading)

    if content and not content.endswith("\n"):
        content += "\n"

    if keep_heading:
        new_lines = lines[:content_start] + [content] + lines[end_idx:]
    else:
        new_lines = lines[:start_idx] + [content] + lines[end_idx:]

    result = frontmatter + "".join(new_lines)

    if not dry_run:
        filepath.write_text(result, encoding="utf-8")
    return result


def ensure_section(
    filepath: Path,
    heading: str,
    content: str = "",
    *,
    level: int = 2,
    after: Optional[str] = None,
    dry_run: bool = False,
) -> str:
    """Ensure a section with *heading* exists.

    If the section already exists, leave it untouched.  If it does not exist,
    create it.  When *after* is given, place it after that section; otherwise
    append at the end of the document.

    Returns the updated full text.
    """
    text = filepath.read_text(encoding="utf-8")
    frontmatter, body = _split_frontmatter(text)
    lines = body.splitlines(keepends=True)
    plain_lines = [l.rstrip("\n").rstrip("\r") for l in lines]

    # Check if heading already exists
    try:
        _find_section(plain_lines, heading)
        return text  # already exists, no-op
    except ValueError:
        pass

    heading_line = f"{'#' * level} {heading}\n"
    block = heading_line
    if content:
        if not content.endswith("\n"):
            content += "\n"
        block += content

    if after:
        _, _, end_idx = _find_section(plain_lines, after)
        new_lines = lines[:end_idx] + ["\n", block] + lines[end_idx:]
    else:
        # Append to end
        if lines and lines[-1].strip():
            new_lines = lines + ["\n", block]
        else:
            new_lines = lines + [block]

    result = frontmatter + "".join(new_lines)

    if not dry_run:
        filepath.write_text(result, encoding="utf-8")
    return result


def append_bullet(
    filepath: Path,
    heading: str,
    bullet: str,
    *,
    dedupe: bool = False,
    dry_run: bool = False,
) -> str:
    """Append a bullet item to the section identified by *heading*.

    If *dedupe* is True, skip appending when an identical bullet already exists
    in that section.

    Returns the updated full text.
    """
    text = filepath.read_text(encoding="utf-8")
    frontmatter, body = _split_frontmatter(text)
    lines = body.splitlines(keepends=True)
    plain_lines = [l.rstrip("\n").rstrip("\r") for l in lines]

    _, content_start, end_idx = _find_section(plain_lines, heading)

    # Normalize bullet
    bullet_stripped = bullet.strip()
    if not bullet_stripped.startswith("- "):
        bullet_stripped = f"- {bullet_stripped}"

    if dedupe:
        for i in range(content_start, end_idx):
            if plain_lines[i].strip() == bullet_stripped:
                return text  # already exists

    bullet_line = bullet_stripped + "\n"
    new_lines = lines[:end_idx] + [bullet_line] + lines[end_idx:]
    result = frontmatter + "".join(new_lines)

    if not dry_run:
        filepath.write_text(result, encoding="utf-8")
    return result
