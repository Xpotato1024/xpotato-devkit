from pathlib import Path
from typing import Tuple, Optional, List

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
        start_idx = -1
        for i, line in enumerate(lines):
            if heading in line and line.strip().startswith('#'):
                start_idx = i
                break
                
        if start_idx == -1:
            raise ValueError(f"Heading '{heading}' not found.")
            
        end_idx = _find_heading_end(lines, start_idx, heading)
        return start_idx, end_idx
        
    if function:
        start_idx = -1
        for i, line in enumerate(lines):
            if function in line and any(kw in line for kw in ["def ", "fn ", "class ", "func "]):
                start_idx = i
                break
        
        if start_idx == -1:
            raise ValueError(f"Function/Class '{function}' not found.")
        
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
