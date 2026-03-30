import typer
import glob
from pathlib import Path
from typing import List, Optional
from rich.console import Console
from rich.table import Table

from devkit.core.encoding import check_encoding
from devkit.core.config import load_config

app = typer.Typer()
console = Console()

@app.command("check")
def check(
    files: List[str] = typer.Argument(..., help="Files or glob patterns to check encoding for"),
):
    """Check text files for UTF-8 validity, BOM, and other anomalies."""
    try:
        config = load_config()
        ignore_patterns = config.get("encoding", {}).get("ignore", [".git", "node_modules", "__pycache__", ".venv", "venv"])
    except Exception as e:
        console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(code=1)

    table = Table(title="Encoding Check Results")
    table.add_column("File", style="cyan")
    table.add_column("Valid UTF-8", style="green")
    table.add_column("BOM", style="yellow")
    table.add_column("Replacement Char", style="red")
    table.add_column("Control Chars", style="red")
    table.add_column("Mixed Newlines", style="magenta")
    
    has_errors = False

    all_files = []
    for f in files:
        if "*" in f or "?" in f:
            all_files.extend(glob.glob(f, recursive=True))
        else:
            all_files.append(f)

    if not all_files:
        console.print("[yellow]No files matched the input patterns.[/yellow]")
        raise typer.Exit(code=1)

    processed_files = 0
    for filename in all_files:
        path = Path(filename)
        if not path.is_file():
            continue
            
        # Ignore logic
        should_ignore = False
        parts = path.parts
        for pattern in ignore_patterns:
            if pattern in parts or path.match(pattern):
                should_ignore = True
                break
        
        if should_ignore:
            continue

        processed_files += 1

        res = check_encoding(path)
        
        issues = (
            not res["valid_utf8"] or 
            res["has_bom"] or 
            res["has_replacement_char"] or 
            res["has_control_chars"] or 
            res["mixed_newlines"]
        )
        if issues:
            has_errors = True
            
        table.add_row(
            res["file"][:50] + "..." if len(res["file"]) > 53 else res["file"],
            "[green]Yes[/green]" if res["valid_utf8"] else f"[red]No[/red]",
            "[red]Yes[/red]" if res["has_bom"] else "[green]No[/green]",
            "[red]Yes[/red]" if res["has_replacement_char"] else "[green]No[/green]",
            "[red]Yes[/red]" if res["has_control_chars"] else "[green]No[/green]",
            "[red]Yes[/red]" if res["mixed_newlines"] else "[green]No[/green]",
        )
        
    if processed_files == 0:
        console.print("[yellow]No valid files found to process.[/yellow]")
        raise typer.Exit(code=1)
        
    console.print(table)
    if has_errors:
        raise typer.Exit(code=1)

@app.command("normalize", hidden=True)
def normalize(
    files: List[str] = typer.Argument(..., help="Files or glob patterns to normalize"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Show what would be done without making changes"),
):
    """(Stub) Normalize files (remove BOM, standardize newlines to LF)."""
    raise typer.Exit("Normalize is not fully implemented yet in Phase 1.")
