import subprocess
from dataclasses import dataclass
from typing import Any, Dict, List, Optional

from devkit.core.timing import timed


def run_git_command(args: List[str]) -> str:
    with timed("git"):
        result = subprocess.run(["git"] + args, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        error = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(f"Git command failed: {error}")
    return result.stdout.strip()


@dataclass(frozen=True)
class DiffScope:
    mode: str
    description: str
    diff_args: List[str]
    refspec: Optional[str] = None


def build_diff_scope(
    staged: bool = False,
    base: Optional[str] = None,
    head: Optional[str] = None,
    commits: Optional[str] = None,
) -> DiffScope:
    if commits and (staged or base or head):
        raise ValueError("`--commits` cannot be combined with `--staged` or `--base/--head`.")
    if staged and (base or head):
        raise ValueError("`--staged` cannot be combined with `--base/--head`.")
    if (base and not head) or (head and not base):
        raise ValueError("`--base` and `--head` must be provided together.")

    if commits:
        return DiffScope(
            mode="commits",
            description=f"commit range {commits}",
            diff_args=["diff", "--numstat", commits],
            refspec=commits,
        )
    if base and head:
        refspec = f"{base}...{head}"
        return DiffScope(
            mode="range",
            description=f"range {base}...{head}",
            diff_args=["diff", "--numstat", refspec],
            refspec=refspec,
        )
    if staged:
        return DiffScope(
            mode="staged",
            description="staged changes",
            diff_args=["diff", "--numstat", "--staged"],
        )
    return DiffScope(
        mode="unstaged",
        description="unstaged changes",
        diff_args=["diff", "--numstat"],
    )


def summarize_diff(
    staged: bool = False,
    base: Optional[str] = None,
    head: Optional[str] = None,
    commits: Optional[str] = None,
) -> Dict[str, Any]:
    """Summarize a git diff using git diff --numstat."""
    scope = build_diff_scope(staged=staged, base=base, head=head, commits=commits)
    return summarize_diff_scope(scope)


def summarize_diff_scope(scope: DiffScope) -> Dict[str, Any]:
    """Summarize a pre-resolved git diff scope using git diff --numstat."""
    output = run_git_command(scope.diff_args)

    files_changed = []
    total_additions = 0
    total_deletions = 0

    if output:
        for line in output.split("\n"):
            if not line:
                continue
            parts = line.split("\t")
            if len(parts) < 3:
                continue
            adds = parts[0]
            dels = parts[1]
            fname = parts[2]

            adds_count = int(adds) if adds != "-" else 0
            dels_count = int(dels) if dels != "-" else 0

            total_additions += adds_count
            total_deletions += dels_count

            files_changed.append(
                {
                    "path": fname,
                    "additions": adds_count,
                    "deletions": dels_count,
                    "is_binary": adds == "-",
                }
            )

    return {
        "scope": {
            "mode": scope.mode,
            "description": scope.description,
            "refspec": scope.refspec,
        },
        "files": files_changed,
        "total_additions": total_additions,
        "total_deletions": total_deletions,
    }
