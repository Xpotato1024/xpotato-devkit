"""CLI command for compact project tree display."""

import typer
from pathlib import Path
from typing import Optional, List
from rich.console import Console

from devkit.core.config import load_config
from devkit.core.tree import format_tree, scan_tree, tree_summary

app = typer.Typer()
console = Console()


@app.callback(invoke_without_command=True)
def tree_cmd(
    ctx: typer.Context,
    path: Optional[Path] = typer.Option(None, "--path", help="Root directory to scan (default: current directory)"),
    max_depth: Optional[int] = typer.Option(None, "--max-depth", help="Maximum depth to descend"),
    ext: Optional[str] = typer.Option(None, "--ext", help="Comma-separated list of extensions to include (e.g. '.py,.rs')"),
    dirs_only: bool = typer.Option(False, "--dirs-only", help="Show only directories"),
    no_gitignore: bool = typer.Option(False, "--no-gitignore", help="Do not read .gitignore"),
):
    """Display a compact project tree, respecting .gitignore and devkit.toml ignore patterns."""
    if ctx.invoked_subcommand is not None:
        return

    root = (path or Path.cwd()).resolve()
    if not root.is_dir():
        console.print(f"[red]Error:[/red] {root} is not a directory.")
        raise typer.Exit(1)

    # Load extra ignores from devkit.toml
    try:
        config = load_config(cwd=root)
        extra_ignore = config.get("encoding", {}).get("ignore", [])
    except Exception:
        extra_ignore = []

    extensions = None
    if ext:
        extensions = set()
        for e in ext.split(","):
            e = e.strip()
            if not e.startswith("."):
                e = "." + e
            extensions.add(e.lower())

    entry = scan_tree(
        root,
        max_depth=max_depth,
        extensions=extensions,
        dirs_only=dirs_only,
        use_gitignore=not no_gitignore,
        extra_ignore=extra_ignore,
    )

    lines = format_tree(entry)
    for line in lines:
        print(line)
    print(tree_summary(entry))
