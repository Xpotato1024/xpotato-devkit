"""CLI commands for patch application and diagnostics."""

import typer
from pathlib import Path
from rich.console import Console
from rich.table import Table

from devkit.core.patch import apply_patch, diagnose_patch, parse_patch_hunks

app = typer.Typer()
console = Console()


@app.command("apply")
def apply_cmd(
    patch_file: Path = typer.Option(..., "--patch-file", help="Path to the unified diff patch file"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Check if the patch applies cleanly without modifying files"),
    verbose: bool = typer.Option(False, "--verbose", help="Show detailed per-hunk information"),
    reject: bool = typer.Option(False, "--reject", help="Apply as much as possible, write .rej files for failed hunks"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Apply a unified diff patch to the filesystem."""
    if not patch_file.is_file():
        console.print(f"[red]Error:[/red] Patch file {patch_file} does not exist.")
        raise typer.Exit(1)

    diag = apply_patch(patch_file, dry_run=dry_run, verbose=verbose, reject=reject)

    if diag.success:
        if brief:
            if dry_run:
                print(f"OK: patch applies cleanly ({diag.total_hunks} hunks)")
            else:
                print(f"OK: {diag.applied_hunks} hunks applied to {len(diag.affected_files)} files")
            return
        if dry_run:
            console.print(f"[green]Patch applies cleanly ({diag.total_hunks} hunk(s)).[/green]")
        else:
            mode = "with --reject" if reject else ""
            console.print(f"[green]Successfully applied patch {patch_file} {mode}[/green]")
            console.print(f"  {diag.applied_hunks} hunk(s) across {len(diag.affected_files)} file(s)")
    else:
        if brief:
            reason = diag.errors[0] if diag.errors else "unknown"
            print(f"FAIL: {diag.failed_hunks}/{diag.total_hunks} hunks failed ({reason})")
            raise typer.Exit(1)

        console.print(f"[red]Patch failed to apply.[/red]")
        console.print(f"  {diag.failed_hunks}/{diag.total_hunks} hunk(s) failed")

        if diag.affected_files:
            console.print(f"  Affected files: {', '.join(diag.affected_files)}")

        if diag.errors:
            console.print()
            console.print("[bold]Error details:[/bold]")
            for err in diag.errors[:10]:
                console.print(f"  [dim]{err}[/dim]")
            if len(diag.errors) > 10:
                console.print(f"  ... and {len(diag.errors) - 10} more")

        console.print()
        console.print("[bold]Compact summary (for re-generation):[/bold]")
        console.print(diag.summary())
        raise typer.Exit(1)


@app.command("diagnose")
def diagnose_cmd(
    patch_file: Path = typer.Option(..., "--patch-file", help="Path to the unified diff patch file"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Diagnose a patch file without applying it.

    Parses the patch, runs a dry-run check, and reports hunk-level details.
    """
    if not patch_file.is_file():
        console.print(f"[red]Error:[/red] Patch file {patch_file} does not exist.")
        raise typer.Exit(1)

    patch_text = patch_file.read_text(encoding="utf-8", errors="replace")
    hunks, files = parse_patch_hunks(patch_text)

    # Hunk table
    if hunks:
        table = Table(title="Patch Hunks")
        table.add_column("File", style="cyan")
        table.add_column("Old", justify="right")
        table.add_column("New", justify="right")
        table.add_column("Header", style="dim")

        for h in hunks:
            table.add_row(
                h.file,
                f"L{h.old_start}+{h.old_count}",
                f"L{h.new_start}+{h.new_count}",
                h.header[:60],
            )
        console.print(table)
    else:
        console.print("[yellow]No hunks found in patch file.[/yellow]")

    # Dry-run check
    diag = diagnose_patch(patch_file)

    if brief:
        if diag.success:
            print(f"OK: {len(hunks)} hunks, {len(files)} files, will apply cleanly")
        else:
            errors = "; ".join(diag.errors[:3])
            print(f"FAIL: {diag.failed_hunks}/{diag.total_hunks} hunks conflict ({errors})")
            raise typer.Exit(1)
        return

    console.print()

    if diag.success:
        console.print(f"[green]Diagnosis: patch will apply cleanly.[/green]")
    else:
        console.print(f"[red]Diagnosis: patch will NOT apply cleanly.[/red]")
        console.print()
        console.print(diag.summary())
        raise typer.Exit(1)
