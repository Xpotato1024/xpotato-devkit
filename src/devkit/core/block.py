from pathlib import Path
import difflib
import re
from typing import List, Optional, Tuple


FUNCTION_PATTERNS = (
    re.compile(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("),
    re.compile(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)\b"),
    re.compile(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("),
    re.compile(r"^\s*func\s+(?:\([^)]+\)\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*\("),
)


def list_markdown_headings(filepath: Path) -> List[dict[str, object]]:
    entries: List[dict[str, object]] = []
    for line_number, line in enumerate(filepath.read_text(encoding="utf-8").splitlines(), start=1):
        stripped = line.strip()
        if not stripped.startswith("#"):
            continue
        marker = stripped.split()[0]
        if set(marker) != {"#"}:
            continue
        entries.append({"line": line_number, "level": len(marker), "text": stripped[len(marker):].strip()})
    return entries


def list_functions(filepath: Path) -> List[dict[str, object]]:
    entries: List[dict[str, object]] = []
    for line_number, line in enumerate(filepath.read_text(encoding="utf-8").splitlines(), start=1):
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

def _find_heading_end(lines: List[str], start_idx: int, heading: str) -> int:
    """Finds the end of a markdown heading section."""
    # The heading must start with '#'
    level_match = heading.strip().split()[0] if heading.strip().startswith('#') else None
    
    for i in range(start_idx + 1, len(lines)):
        line = lines[i].strip()
        if not line.startswith('#'):
            continue
        # If it's a heading of same or higher level, the section ends here
        if level_match and line.startswith('#') and len(line.split()[0]) <= len(level_match):
            return i
    return len(lines)

def find_block_bounds(
    lines: List[str], 
    line_range: Optional[str] = None, 
    marker: Optional[str] = None,
    heading: Optional[str] = None,
    function: Optional[str] = None,
) -> Tuple[int, int]:
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
            
    if marker:
        start_idx = -1
        end_idx = -1
        for i, line in enumerate(lines):
            if marker in line:
                if start_idx == -1:
                    start_idx = i
                else:
                    end_idx = i + 1  # Include the closing marker
                    break
        
        if start_idx == -1:
            raise ValueError(f"Marker '{marker}' not found.")
        if end_idx == -1:
            end_idx = len(lines) # Up to end of file if no closing marker found
            
        return start_idx, end_idx
        
    if heading:
        available_headings = [line.strip() for line in lines if line.strip().startswith("#")]
        start_idx = -1
        for i, line in enumerate(lines):
            if heading in line and line.strip().startswith('#'):
                start_idx = i
                break
                
        if start_idx == -1:
            suggestions = suggest_candidates(heading, available_headings)
            message = f"Heading '{heading}' not found."
            if suggestions:
                message += f" Did you mean: {', '.join(suggestions)}?"
            raise ValueError(message)
            
        end_idx = _find_heading_end(lines, start_idx, heading)
        return start_idx, end_idx
        
    if function:
        available_functions = []
        start_idx = -1
        for i, line in enumerate(lines):
            for pattern in FUNCTION_PATTERNS:
                match = pattern.search(line)
                if not match:
                    continue
                available_functions.append(match.group(1))
                if function == match.group(1):
                    start_idx = i
                    break
            if start_idx != -1:
                break
        
        if start_idx == -1:
            suggestions = suggest_candidates(function, available_functions)
            message = f"Function/Class '{function}' not found."
            if suggestions:
                message += f" Did you mean: {', '.join(suggestions)}?"
            raise ValueError(message)
        
        end_idx = start_idx + 1
        while end_idx < len(lines):
            if not lines[end_idx].strip():
                # End of function is assumed at first empty line (heuristic)
                break
            end_idx += 1
            
        return start_idx, end_idx
        
    raise ValueError("Must specify line_range, marker, heading, or function.")

def extract_block(
    filepath: Path,
    line_range: Optional[str] = None,
    marker: Optional[str] = None,
    heading: Optional[str] = None,
    function: Optional[str] = None,
) -> str:
    lines = filepath.read_text(encoding="utf-8").splitlines(keepends=True)
    start, end = find_block_bounds(lines, line_range, marker, heading, function)
    return "".join(lines[start:end])

def replace_block(
    filepath: Path,
    replacement: str,
    line_range: Optional[str] = None,
    marker: Optional[str] = None,
    heading: Optional[str] = None,
    function: Optional[str] = None,
    dry_run: bool = False
) -> Tuple[str, str]:
    lines = filepath.read_text(encoding="utf-8").splitlines(keepends=True)
    start, end = find_block_bounds(lines, line_range, marker, heading, function)
    
    old_block = "".join(lines[start:end])
    
    if replacement and not replacement.endswith('\n'):
        replacement += '\n'
        
    new_lines = lines[:start] + [replacement] + lines[end:]
    
    if not dry_run:
        filepath.write_text("".join(new_lines), encoding="utf-8")
        
    return old_block, replacement
