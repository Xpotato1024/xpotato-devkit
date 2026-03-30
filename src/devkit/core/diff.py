import subprocess
from typing import Dict, Any, List

def run_git_command(args: List[str]) -> str:
    result = subprocess.run(["git"] + args, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        raise RuntimeError(f"Git command failed: {result.stderr.strip()}")
    return result.stdout.strip()

def summarize_diff(staged: bool = False) -> Dict[str, Any]:
    """Summarize the diff using git diff --numstat."""
    args = ["diff", "--numstat"]
    if staged:
        args.append("--staged")
    
    output = run_git_command(args)
    
    files_changed = []
    total_additions = 0
    total_deletions = 0
    
    if not output:
        return {
            "files": [],
            "total_additions": 0,
            "total_deletions": 0
        }
        
    for line in output.split('\n'):
        if not line:
            continue
        parts = line.split('\t')
        if len(parts) >= 3:
            adds = parts[0]
            dels = parts[1]
            fname = parts[2]
            
            # binary files show '-' for numstat
            adds_count = int(adds) if adds != '-' else 0
            dels_count = int(dels) if dels != '-' else 0
            
            total_additions += adds_count
            total_deletions += dels_count
            
            files_changed.append({
                "path": fname,
                "additions": adds_count,
                "deletions": dels_count,
                "is_binary": adds == '-'
            })
            
    return {
        "files": files_changed,
        "total_additions": total_additions,
        "total_deletions": total_deletions
    }
