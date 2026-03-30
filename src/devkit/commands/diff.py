import typer
from rich.console import Console
from rich.table import Table
import json
from devkit.core.diff import summarize_diff

app = typer.Typer()
console = Console()

@app.command("summarize")
def summarize(
    staged: bool = typer.Option(False, "--staged", help="Summarize staged changes instead of unstaged"),
    output_json: bool = typer.Option(False, "--json", help="Output in JSON format"),
):
    """Summarize changes in the working directory or staging area."""
    try:
        summary = summarize_diff(staged=staged)
    except RuntimeError as e:
        console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(code=1)
        
    if output_json:
        console.print(json.dumps(summary, indent=2))
        return
        
    if not summary["files"]:
        console.print("[yellow]No changes found.[/yellow]")
        return
        
    table = Table(title=f"Diff Summary ({'staged' if staged else 'unstaged'})")
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
