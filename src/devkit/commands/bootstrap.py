from pathlib import Path
from typing import Optional

import typer
from rich.console import Console

from devkit.bootstrap import bootstrap_self, find_repo_root

app = typer.Typer()
console = Console()


@app.command("install-self")
def install_self(
    repo_root: Optional[Path] = typer.Option(
        None,
        "--repo-root",
        help="Path to the devkit repository root. Defaults to auto-detection from the current directory.",
    ),
):
    """Install the current devkit checkout as a user tool via uv."""
    try:
        resolved_root = repo_root.resolve() if repo_root else find_repo_root()
        tool_bin = bootstrap_self(resolved_root)
    except Exception as exc:
        console.print(f"[red]Error:[/red] {exc}")
        raise typer.Exit(1)

    console.print(
        "[green]Bootstrap complete.[/green] "
        f"If the current shell does not see devkit yet, restart it or add {tool_bin} to PATH."
    )
