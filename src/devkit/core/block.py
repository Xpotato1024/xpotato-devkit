from pathlib import Path
import difflib
import re
import unicodedata
from typing import List, Optional, Tuple
from devkit.core.timing import timed


# ---------------------------------------------------------------------------
# Language patterns
# ---------------------------------------------------------------------------

FUNCTION_PATTERNS = (
    # Python
    re.compile(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("),
    re.compile(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    # Rust
    re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]"),
    re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|mod|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"^\s*impl(?:<[^>]+>)?\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    # Go
    re.compile(r"^\s*func\s+(?:\([^)]+\)\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*\("),
    # JavaScript / TypeScript
    re.compile(r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*[<(]"),
    re.compile(r"^\s*(?:export\s+)?class\s+([A-Za-z_$][A-Za-z0-9_$]*)\b"),
)

# Map file extensions to language families for end-of-block detection.
_INDENT_LANGS = frozenset({".py"})
_BRACE_LANGS = frozenset({
    ".rs", ".go", ".c", ".cpp", ".h", ".hpp", ".cc",
    ".js", ".jsx", ".ts", ".tsx", ".java", ".cs", ".kt", ".swift",
})


# ---------------------------------------------------------------------------
# Heading utilities
# ---------------------------------------------------------------------------

def _slugify(text: str) -> str:
    """Convert a heading text into a URL-friendly slug."""
    text = unicodedata.normalize("NFKD", text)
    text = text.lower()
    text = re.sub(r"[^\w\s-]", "", text)
    text = re.sub(r"[\s_]+", "-", text).strip("-")
    return text


def list_markdown_headings(filepath: Path) -> List[dict[str, object]]:
    entries: List[dict[str, object]] = []
    with timed("io"):
        content = filepath.read_text(encoding="utf-8")
    for line_number, line in enumerate(content.splitlines(), start=1):
        stripped = line.strip()
        if not stripped.startswith("#"):
            continue
        marker = stripped.split()[0]
        if set(marker) != {"#"}:
            continue
        text = stripped[len(marker):].strip()
        entries.append({
            "line": line_number,
            "level": len(marker),
            "text": text,
            "slug": _slugify(text),
        })
    return entries


def list_functions(filepath: Path) -> List[dict[str, object]]:
    entries: List[dict[str, object]] = []
    with timed("io"):
        content = filepath.read_text(encoding="utf-8")
    for line_number, line in enumerate(content.splitlines(), start=1):
        for pattern in FUNCTION_PATTERNS:
            match = pattern.search(line)
            if match:
                entries.append({"line": line_number, "name": match.group(1)})
                break
    return entries


def suggest_candidates(target: str, choices: List[str], limit: int = 3) -> List[str]:
    if not target or not choices:
        return []
    return difflib.get_close_matches(target, choices, n=limit, cutoff=0.4)


# ---------------------------------------------------------------------------
# End-of-block detection helpers
# ---------------------------------------------------------------------------

def _find_function_end_python(lines: List[str], start_idx: int) -> int:
    """Indentation-based: find end of a Python def/class block."""
    start_line = lines[start_idx].rstrip("\n").rstrip("\r")
    start_indent = len(start_line) - len(start_line.lstrip())
    idx = start_idx + 1
    last_content = start_idx
    while idx < len(lines):
        raw = lines[idx].rstrip("\n").rstrip("\r")
        stripped = raw.strip()
        if stripped:  # non-blank line
            cur_indent = len(raw) - len(raw.lstrip())
            # If we reach a line with same or less indentation that is not a
            # comment or decorator, the block has ended.
            if cur_indent <= start_indent and not stripped.startswith(("#", "@", ")")):
                break
            last_content = idx
        idx += 1
    return last_content + 1


def _find_function_end_braces(lines: List[str], start_idx: int) -> int:
    """Brace-tracking: find end of fn/struct/impl block in brace languages."""
    depth = 0
    found_open = False
    for idx in range(start_idx, len(lines)):
        for ch in lines[idx]:
            if ch == "{":
                depth += 1
                found_open = True
            elif ch == "}":
                depth -= 1
                if found_open and depth == 0:
                    return idx + 1
    return len(lines)


def _find_function_end_fallback(lines: List[str], start_idx: int) -> int:
    """Legacy heuristic: end at first empty line after start."""
    idx = start_idx + 1
    while idx < len(lines):
        if not lines[idx].strip():
            break
        idx += 1
    return idx


def _detect_end_strategy(filepath: Optional[Path]):
    """Return the appropriate end-of-block function for the given file."""
    if filepath is None:
        return _find_function_end_fallback
    suffix = filepath.suffix.lower()
    if suffix in _INDENT_LANGS:
        return _find_function_end_python
    if suffix in _BRACE_LANGS:
        return _find_function_end_braces
    return _find_function_end_fallback


# ---------------------------------------------------------------------------
# Heading section boundary
# ---------------------------------------------------------------------------

def _find_heading_end(lines: List[str], start_idx: int, heading_level: int) -> int:
    """Find the end of a markdown heading section (exclusive)."""
    for i in range(start_idx + 1, len(lines)):
        line = lines[i].strip()
        if not line.startswith("#"):
            continue
        parts = line.split()
        if not parts:
            continue
        marker = parts[0]
        if set(marker) != {"#"}:
            continue
        if len(marker) <= heading_level:
            return i
    return len(lines)


def _heading_level(line: str) -> int:
    """Return the heading level for a markdown heading line, or 0."""
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


# ---------------------------------------------------------------------------
# Block bounds detection
# ---------------------------------------------------------------------------

def find_block_bounds(
    lines: List[str],
    line_range: Optional[str] = None,
    marker: Optional[str] = None,
    heading: Optional[str] = None,
    function: Optional[str] = None,
    *,
    heading_exact: bool = False,
    filepath: Optional[Path] = None,
) -> Tuple[int, int]:
    # --- line range ---
    if line_range:
        try:
            start_str, end_str = line_range.split("-")
            start = int(start_str) - 1
            end = int(end_str)
            if start < 0 or start >= len(lines) or end < start or end > len(lines):
                raise ValueError("Line range out of bounds.")
            return start, end
        except Exception:
            raise ValueError(f"Invalid line range: {line_range}")

    # --- marker ---
    if marker:
        start_idx = -1
        end_idx = -1
        for i, line in enumerate(lines):
            if marker in line:
                if start_idx == -1:
                    start_idx = i
                else:
                    end_idx = i + 1
                    break

        if start_idx == -1:
            raise ValueError(f"Marker '{marker}' not found.")
        if end_idx == -1:
            end_idx = len(lines)

        return start_idx, end_idx

    # --- heading ---
    if heading:
        available_headings = [line.strip() for line in lines if line.strip().startswith("#")]

        matches: List[Tuple[int, int]] = []  # (line_idx, heading_level)
        for i, line in enumerate(lines):
            stripped = line.strip()
            if not stripped.startswith("#"):
                continue
            level = _heading_level(stripped)
            if level == 0:
                continue

            heading_text = stripped[level:].strip()
            full_heading = stripped  # e.g. "## Details"

            if heading_exact:
                if heading_text == heading or full_heading == heading:
                    matches.append((i, level))
            else:
                if heading in stripped:
                    matches.append((i, level))

        if not matches:
            suggestions = suggest_candidates(heading, available_headings)
            message = f"Heading '{heading}' not found."
            if suggestions:
                message += f" Did you mean: {', '.join(suggestions)}?"
            raise ValueError(message)

        if len(matches) > 1:
            locations = ", ".join(f"L{m[0]+1}" for m in matches)
            raise ValueError(
                f"Heading '{heading}' matched {len(matches)} sections ({locations}). "
                f"Use --lines or --heading-exact to disambiguate."
            )

        start_idx, level = matches[0]
        end_idx = _find_heading_end(lines, start_idx, level)
        return start_idx, end_idx

    # --- function / symbol ---
    if function:
        available_functions: List[str] = []
        start_idx = -1
        for i, line in enumerate(lines):
            for pattern in FUNCTION_PATTERNS:
                match = pattern.search(line)
                if not match:
                    continue
                available_functions.append(match.group(1))
                if function == match.group(1) and start_idx == -1:
                    start_idx = i
                break

        if start_idx == -1:
            suggestions = suggest_candidates(function, available_functions)
            message = f"Function/Class '{function}' not found."
            if suggestions:
                message += f" Did you mean: {', '.join(suggestions)}?"
            raise ValueError(message)

        end_strategy = _detect_end_strategy(filepath)
        end_idx = end_strategy(lines, start_idx)
        return start_idx, end_idx

    raise ValueError("Must specify line_range, marker, heading, or function.")


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def extract_block(
    filepath: Path,
    line_range: Optional[str] = None,
    marker: Optional[str] = None,
    heading: Optional[str] = None,
    function: Optional[str] = None,
    *,
    heading_exact: bool = False,
) -> str:
    with timed("io"):
        content = filepath.read_text(encoding="utf-8")
    lines = content.splitlines(keepends=True)
    start, end = find_block_bounds(
        lines, line_range, marker, heading, function,
        heading_exact=heading_exact, filepath=filepath,
    )
    return "".join(lines[start:end])


def replace_block(
    filepath: Path,
    replacement: str,
    line_range: Optional[str] = None,
    marker: Optional[str] = None,
    heading: Optional[str] = None,
    function: Optional[str] = None,
    dry_run: bool = False,
    *,
    heading_exact: bool = False,
) -> Tuple[str, str]:
    with timed("io"):
        content = filepath.read_text(encoding="utf-8")
    lines = content.splitlines(keepends=True)
    start, end = find_block_bounds(
        lines, line_range, marker, heading, function,
        heading_exact=heading_exact, filepath=filepath,
    )

    old_block = "".join(lines[start:end])

    if replacement and not replacement.endswith("\n"):
        replacement += "\n"

    new_lines = lines[:start] + [replacement] + lines[end:]

    if not dry_run:
        with timed("io"):
            filepath.write_text("".join(new_lines), encoding="utf-8")

    return old_block, replacement


def diff_preview(old_block: str, new_block: str, filepath: Path | str = "file") -> str:
    """Return a unified diff string comparing old and new blocks."""
    old_lines = old_block.splitlines(keepends=True)
    new_lines = new_block.splitlines(keepends=True)
    fname = str(filepath)
    diff = difflib.unified_diff(old_lines, new_lines, fromfile=fname, tofile=fname)
    return "".join(diff)


# ---------------------------------------------------------------------------
# Outline (signature-only extraction)
# ---------------------------------------------------------------------------

_IMPORT_RE = re.compile(
    r"^\s*(?:import |from \S+ import |use |require\(|#include |"
    r"const \S+ = require\(|package )"
)

_DECORATOR_RE = re.compile(r"^\s*@")

_DOCSTRING_OPEN_RE = re.compile(r'^\s*(?:"""|\'\'\'|///|/\*\*)')


def outline_file(
    filepath: Path,
    *,
    include_imports: bool = False,
    include_docstrings: bool = False,
) -> List[str]:
    """Extract function/class signatures from a file.

    Returns a list of formatted lines: ``LN: <signature>``
    """
    with timed("io"):
        content = filepath.read_text(encoding="utf-8")
    lines = content.splitlines()
    result: List[str] = []

    # Imports
    if include_imports:
        for i, line in enumerate(lines):
            if _IMPORT_RE.match(line):
                result.append(f"L{i+1}: {line.rstrip()}")
        if result:
            result.append("")  # separator

    # Functions/classes — include decorator lines
    i = 0
    while i < len(lines):
        line = lines[i]
        matched = False
        for pattern in FUNCTION_PATTERNS:
            if pattern.search(line):
                matched = True
                break

        if matched:
            # Collect preceding decorators
            decorators: List[str] = []
            j = i - 1
            while j >= 0 and _DECORATOR_RE.match(lines[j]):
                decorators.insert(0, f"L{j+1}: {lines[j].rstrip()}")
                j -= 1

            result.extend(decorators)
            result.append(f"L{i+1}: {line.rstrip()}")

            # Optional: first line of docstring
            if include_docstrings and i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                if _DOCSTRING_OPEN_RE.match(lines[i + 1]) or next_line.startswith("#"):
                    # Single-line docstring or comment
                    doc_line = next_line
                    if doc_line.startswith(('"""', "'''")):
                        # Trim to just the text
                        doc_line = doc_line.strip("\"'").strip()
                    if doc_line:
                        result.append(f"L{i+2}:     # {doc_line}")

        i += 1

    return result


# ---------------------------------------------------------------------------
# Context extraction (symbol + margin)
# ---------------------------------------------------------------------------

def extract_context(
    filepath: Path,
    function: str,
    margin: int = 5,
) -> str:
    """Extract a symbol with surrounding context lines, numbered.

    Returns content with line numbers, suitable for patch context.
    """
    with timed("io"):
        content = filepath.read_text(encoding="utf-8")
    lines = content.splitlines()
    plain = [l.rstrip("\n").rstrip("\r") for l in lines]

    # Find the symbol using find_block_bounds
    raw_lines = filepath.read_text(encoding="utf-8").splitlines(keepends=True)
    start, end = find_block_bounds(
        raw_lines, function=function, filepath=filepath,
    )

    # Expand with margin
    ctx_start = max(0, start - margin)
    ctx_end = min(len(lines), end + margin)

    result_lines: List[str] = []
    result_lines.append(f"--- {filepath} L{ctx_start+1}-{ctx_end} ---")
    for i in range(ctx_start, ctx_end):
        result_lines.append(f"{i+1}: {plain[i]}")

    return "\n".join(result_lines) + "\n"

