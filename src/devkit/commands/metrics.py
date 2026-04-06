import typer
from pathlib import Path
from rich.console import Console
from rich.table import Table

from devkit.core.metrics import get_metrics_file, load_metrics, summarize_metrics

app = typer.Typer(help="Manage devkit metrics.")
console = Console()

@app.command("show")
def show(
    path: Path = typer.Option(None, "--path", help="Path to override devkit.toml derived metrics file"),
):
    """Show local aggregated usage metrics."""
    target = path or get_metrics_file()
    
    if not target:
        console.print("[yellow]Metrics are not enabled in devkit.toml or no path configured.[/yellow]")
        raise typer.Exit()
        
    if not target.is_file():
        console.print(f"[yellow]Metrics file {target} does not exist yet.[/yellow]")
        raise typer.Exit()
        
    records = load_metrics(target)
    if not records:
        console.print(f"[yellow]No valid records found in {target}.[/yellow]")
        raise typer.Exit()
        
    summary = summarize_metrics(records)
    
    table = Table(title=f"Devkit Usage Metrics (Total: {len(records)} runs)")
    table.add_column("Command", style="cyan")
    table.add_column("Count", justify="right", style="green")
    table.add_column("Avg Time (ms)", justify="right", style="yellow")
    table.add_column("Brief %", justify="right")
    table.add_column("Success %", justify="right")
    
    for cmd, st in sorted(summary.items(), key=lambda x: x[1]["count"], reverse=True):
        brief_pct = (st["brief_count"] / st["count"]) * 100
        success_pct = st["success_rate"] * 100
        table.add_row(
            cmd,
            str(st["count"]),
            f"{st['avg_ms']:.1f}",
            f"{brief_pct:.1f}%",
            f"{success_pct:.1f}%"
        )
        
    console.print(table)
