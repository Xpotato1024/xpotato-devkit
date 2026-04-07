"""Core logic for patch application and diagnostics.

Wraps ``git apply`` with enhanced error reporting, partial application
support, and a diagnostic report for AI-friendly error summaries.
"""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------

@dataclass(frozen=True)
class HunkInfo:
    """Represents a single hunk from a unified diff patch."""
    file: str
    old_start: int
    old_count: int
    new_start: int
    new_count: int
    header: str


@dataclass
class PatchDiagnostic:
    """Diagnostic report for a patch application attempt."""
    success: bool
    total_hunks: int = 0
    applied_hunks: int = 0
    failed_hunks: int = 0
    errors: List[str] = field(default_factory=list)
    affected_files: List[str] = field(default_factory=list)

    def summary(self) -> str:
        """Return a compact error summary for AI re-generation."""
        if self.success:
            return f"Patch applied cleanly ({self.total_hunks} hunk(s), {len(self.affected_files)} file(s))."

        lines = [
            f"Patch FAILED: {self.failed_hunks}/{self.total_hunks} hunk(s) failed.",
            f"Affected files: {', '.join(self.affected_files) or 'unknown'}",
        ]
        for err in self.errors[:5]:
            lines.append(f"  - {err}")
        if len(self.errors) > 5:
            lines.append(f"  - ... and {len(self.errors) - 5} more error(s)")
        return "\n".join(lines)


# ---------------------------------------------------------------------------
# Patch parsing
# ---------------------------------------------------------------------------

_HUNK_RE = re.compile(
    r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@"
)
_DIFF_FILE_RE = re.compile(r"^(?:---|\+\+\+) [ab]/(.+)")


def parse_patch_hunks(patch_text: str) -> tuple[List[HunkInfo], List[str]]:
    """Parse a unified diff and return (hunks, affected_files)."""
    hunks: List[HunkInfo] = []
    files: List[str] = []
    current_file = "unknown"

    for line in patch_text.splitlines():
        file_match = _DIFF_FILE_RE.match(line)
        if file_match:
            fname = file_match.group(1)
            if fname != "/dev/null" and fname not in files:
                current_file = fname
                files.append(fname)
            continue

        hunk_match = _HUNK_RE.match(line)
        if hunk_match:
            hunks.append(HunkInfo(
                file=current_file,
                old_start=int(hunk_match.group(1)),
                old_count=int(hunk_match.group(2) or "1"),
                new_start=int(hunk_match.group(3)),
                new_count=int(hunk_match.group(4) or "1"),
                header=line,
            ))

    return hunks, files


# ---------------------------------------------------------------------------
# Application
# ---------------------------------------------------------------------------

def apply_patch(
    patch_file: Path,
    dry_run: bool = False,
    verbose: bool = False,
    reject: bool = False,
) -> PatchDiagnostic:
    """Apply a patch file using ``git apply``.

    Parameters
    ----------
    patch_file : Path
        Path to the unified diff file.
    dry_run : bool
        If True, only check whether the patch applies cleanly.
    verbose : bool
        If True (or on failure), include per-hunk detail in the diagnostic.
    reject : bool
        If True, apply as much as possible and write ``.rej`` files for
        failed hunks (uses ``git apply --reject``).

    Returns
    -------
    PatchDiagnostic
        A diagnostic report describing the outcome.
    """
    patch_text = patch_file.read_text(encoding="utf-8", errors="replace")
    hunks, affected_files = parse_patch_hunks(patch_text)

    args = ["git", "apply"]
    if dry_run:
        args.append("--check")
    if reject:
        args.append("--reject")
    if verbose:
        args.append("--verbose")
    args.append(str(patch_file))

    result = subprocess.run(args, capture_output=True, text=True, encoding="utf-8")

    diag = PatchDiagnostic(
        success=result.returncode == 0,
        total_hunks=len(hunks),
        affected_files=affected_files,
    )

    if result.returncode == 0:
        diag.applied_hunks = len(hunks)
    else:
        error_text = result.stderr.strip() or result.stdout.strip()
        diag.errors = [line for line in error_text.splitlines() if line.strip()]
        # Count failed hunks from error output
        fail_count = sum(1 for e in diag.errors if "patch does not apply" in e.lower() or "hunk" in e.lower())
        diag.failed_hunks = max(fail_count, 1)  # at least 1 if failed
        diag.applied_hunks = max(0, len(hunks) - diag.failed_hunks)

    return diag


def diagnose_patch(patch_file: Path) -> PatchDiagnostic:
    """Run a dry-run check and return a diagnostic without modifying files."""
    return apply_patch(patch_file, dry_run=True, verbose=True)
