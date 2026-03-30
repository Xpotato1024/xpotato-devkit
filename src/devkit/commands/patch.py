import typer
from pathlib import Path
from rich.console import Console

from devkit.core.patch import apply_patch

app = typer.Typer()
console = Console()

@app.command("apply")
def apply_cmd(
    patch_file: Path = typer.Option(..., "--patch-file", help="Path to the unified diff patch file"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Check if the patch applies cleanly without modifying files")
):
    """Apply a unified diff patch to the filesystem."""
    if not patch_file.is_file():
        console.print(f"[red]Error:[/red] Patch file {patch_file} does not exist.")
        raise typer.Exit(1)
        
    try:
        apply_patch(patch_file, dry_run=dry_run)
    except RuntimeError as e:
        console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)
        
    if dry_run:
        console.print("[green]Patch applies cleanly.[/green]")
    else:
        console.print(f"[green]Successfully applied patch {patch_file}[/green]")
