import typer
from pathlib import Path
from typing import Optional
from rich.console import Console

from devkit.core.block import extract_block, list_functions, list_markdown_headings, replace_block

app = typer.Typer()
console = Console()

@app.command("extract")
def extract(
    file: Path = typer.Argument(..., help="Target file"),
    lines: Optional[str] = typer.Option(None, "--lines", help="Line range e.g. 10-20"),
    marker: Optional[str] = typer.Option(None, "--marker", help="Marker string (extracts between 1st and 2nd occurrence, or EOF if no 2nd)"),
    heading: Optional[str] = typer.Option(None, "--heading", help="Markdown heading to extract e.g. '## 1.'"),
    function: Optional[str] = typer.Option(None, "--function", help="Function name to extract (best-effort heuristic, stops at empty line)"),
    list_headings: bool = typer.Option(False, "--list-headings", help="List Markdown headings in the file"),
    list_functions_flag: bool = typer.Option(False, "--list-functions", help="List best-effort function/class names in the file"),
    output: Optional[Path] = typer.Option(None, "--output", help="Output file path"),
):
    """Extract a block of text from a file."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    if list_headings and list_functions_flag:
        console.print("[red]Error:[/red] Use either --list-headings or --list-functions, not both.")
        raise typer.Exit(1)

    if list_headings or list_functions_flag:
        if any([lines, marker, heading, function, output]):
            console.print("[red]Error:[/red] Listing options cannot be combined with extract selectors or --output.")
            raise typer.Exit(1)
        entries = list_markdown_headings(file) if list_headings else list_functions(file)
        if not entries:
            console.print("[yellow]No entries found.[/yellow]")
            return
        for entry in entries:
            if list_headings:
                print(f"L{entry['line']}: {'#' * int(entry['level'])} {entry['text']}")
            else:
                print(f"L{entry['line']}: {entry['name']}")
        return

    try:
        content = extract_block(file, line_range=lines, marker=marker, heading=heading, function=function)
    except Exception as e:
        console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)

    if output:
        output.write_text(content, encoding="utf-8")
        console.print(f"[green]Extracted block to {output}[/green]")
    else:
        print(content, end="")

@app.command("replace")
def replace(
    file: Path = typer.Argument(..., help="Target file"),
    with_file: Path = typer.Option(..., "--with-file", help="File containing block replacement text"),
    lines: Optional[str] = typer.Option(None, "--lines", help="Line range e.g. 10-20"),
    marker: Optional[str] = typer.Option(None, "--marker", help="Marker string"),
    heading: Optional[str] = typer.Option(None, "--heading", help="Markdown heading"),
    function: Optional[str] = typer.Option(None, "--function", help="Function name"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Dry run, do not modify target file"),
):
    """Replace a block of text in a file."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] Target file {file} does not exist.")
        raise typer.Exit(1)
    if not with_file.is_file():
        console.print(f"[red]Error:[/red] Replacement file {with_file} does not exist.")
        raise typer.Exit(1)
        
    replacement_text = with_file.read_text(encoding="utf-8")
    
    try:
        old_block, new_block = replace_block(
            file, replacement_text, 
            line_range=lines, marker=marker, heading=heading, function=function, 
            dry_run=dry_run
        )
    except Exception as e:
        console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)
        
    if dry_run:
        console.print("[yellow]DRY RUN: file was not modified.[/yellow]")
        console.print(f"Would replace {len(old_block.splitlines())} lines with {len(new_block.splitlines())} lines.")
    else:
        console.print(f"[green]Successfully replaced block in {file}[/green]")
