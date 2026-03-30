import subprocess
from pathlib import Path

def apply_patch(patch_file: Path, dry_run: bool = False) -> None:
    """Applies a patch file using git apply."""
    args = ["git", "apply"]
    if dry_run:
        args.append("--check")
    args.append(str(patch_file))
    
    result = subprocess.run(args, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        error_msg = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"Patch failed to apply:\n{error_msg}")
