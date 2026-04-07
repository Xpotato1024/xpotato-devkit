import typer
from typing import Optional
from rich.console import Console
from rich.table import Table
import json
from devkit.core.diff import summarize_diff

app = typer.Typer()
console = Console()

@app.command("summarize")
def summarize(
    staged: bool = typer.Option(False, "--staged", help="Summarize staged changes instead of unstaged"),
    base: Optional[str] = typer.Option(None, "--base", help="Base ref for an explicit diff range"),
    head: Optional[str] = typer.Option(None, "--head", help="Head ref for an explicit diff range"),
    output_json: bool = typer.Option(False, "--json", help="Output in JSON format"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
    files_only: bool = typer.Option(False, "--files-only", help="Output only file paths, one per line"),
):
    """Summarize changes in the working directory or staging area."""
    try:
        summary = summarize_diff(staged=staged, base=base, head=head)
    except (RuntimeError, ValueError) as e:
        if brief:
            print(f"FAIL: {e}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(code=1)

    if output_json:
        console.print(json.dumps(summary, indent=2))
        return

    scope_description = summary["scope"]["description"]

    if not summary["files"]:
        if brief:
            print(f"OK: 0 files changed ({scope_description})")
        elif files_only:
            pass  # no output
        else:
            console.print(f"[yellow]No changes found for {scope_description}.[/yellow]")
        return

    if files_only:
        for f in summary["files"]:
            print(f["path"])
        return

    if brief:
        n = len(summary["files"])
        print(f"OK: {n} files, +{summary['total_additions']}/-{summary['total_deletions']}")
        return

    table = Table(title=f"Diff Summary ({scope_description})")
    table.add_column("File", style="cyan")
    table.add_column("Additions (+)", style="green", justify="right")
    table.add_column("Deletions (-)", style="red", justify="right")

    for f in summary["files"]:
        adds = str(f["additions"]) if not f["is_binary"] else "binary"
        dels = str(f["deletions"]) if not f["is_binary"] else "binary"
        table.add_row(f["path"], adds, dels)

    table.add_row(
        "[bold]Total[/bold]",
        f"[bold green]{summary['total_additions']}[/]",
        f"[bold red]{summary['total_deletions']}[/]"
    )

    console.print(table)
