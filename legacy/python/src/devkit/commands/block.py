import typer
from pathlib import Path
from typing import Optional
from rich.console import Console
from rich.syntax import Syntax

from devkit.core.block import (
    diff_preview,
    extract_block,
    extract_context,
    list_functions,
    list_markdown_headings,
    outline_file,
    replace_block,
)

app = typer.Typer()
console = Console()

@app.command("extract")
def extract(
    file: Path = typer.Argument(..., help="Target file"),
    lines: Optional[str] = typer.Option(None, "--lines", help="Line range e.g. 10-20"),
    marker: Optional[str] = typer.Option(None, "--marker", help="Marker string (extracts between 1st and 2nd occurrence, or EOF if no 2nd)"),
    heading: Optional[str] = typer.Option(None, "--heading", help="Markdown heading to extract e.g. '## 1.'"),
    heading_exact: bool = typer.Option(False, "--heading-exact", help="Use exact match for heading text (not substring)"),
    function: Optional[str] = typer.Option(None, "--function", help="Function name to extract (language-aware detection)"),
    symbol: Optional[str] = typer.Option(None, "--symbol", help="Symbol name to extract (alias for --function, includes struct/impl/enum)"),
    list_headings: bool = typer.Option(False, "--list-headings", help="List Markdown headings in the file"),
    list_functions_flag: bool = typer.Option(False, "--list-functions", help="List best-effort function/class names in the file"),
    output: Optional[Path] = typer.Option(None, "--output", help="Output file path"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line instead of content"),
):
    """Extract a block of text from a file."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    # --symbol is an alias for --function
    effective_function = symbol or function
    if symbol and function:
        console.print("[red]Error:[/red] Use either --function or --symbol, not both.")
        raise typer.Exit(1)

    if list_headings and list_functions_flag:
        console.print("[red]Error:[/red] Use either --list-headings or --list-functions, not both.")
        raise typer.Exit(1)

    if list_headings or list_functions_flag:
        if any([lines, marker, heading, effective_function, output]):
            console.print("[red]Error:[/red] Listing options cannot be combined with extract selectors or --output.")
            raise typer.Exit(1)
        entries = list_markdown_headings(file) if list_headings else list_functions(file)
        if not entries:
            console.print("[yellow]No entries found.[/yellow]")
            return
        for entry in entries:
            if list_headings:
                slug = entry.get("slug", "")
                print(f"L{entry['line']}: {'#' * int(entry['level'])} {entry['text']}  [{slug}]")
            else:
                print(f"L{entry['line']}: {entry['name']}")
        return

    try:
        content = extract_block(
            file,
            line_range=lines,
            marker=marker,
            heading=heading,
            function=effective_function,
            heading_exact=heading_exact,
        )
    except Exception as e:
        if brief:
            print(f"FAIL: {e}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)

    if brief:
        n_lines = len(content.splitlines())
        print(f"OK: extracted {n_lines} lines from {file}")
        return

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
    heading_exact: bool = typer.Option(False, "--heading-exact", help="Use exact match for heading text"),
    function: Optional[str] = typer.Option(None, "--function", help="Function name"),
    symbol: Optional[str] = typer.Option(None, "--symbol", help="Symbol name (alias for --function)"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Dry run, do not modify target file"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Replace a block of text in a file."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] Target file {file} does not exist.")
        raise typer.Exit(1)
    if not with_file.is_file():
        console.print(f"[red]Error:[/red] Replacement file {with_file} does not exist.")
        raise typer.Exit(1)

    effective_function = symbol or function
    if symbol and function:
        console.print("[red]Error:[/red] Use either --function or --symbol, not both.")
        raise typer.Exit(1)

    replacement_text = with_file.read_text(encoding="utf-8")

    try:
        old_block, new_block = replace_block(
            file, replacement_text,
            line_range=lines, marker=marker, heading=heading, function=effective_function,
            dry_run=dry_run,
            heading_exact=heading_exact,
        )
    except Exception as e:
        if brief:
            print(f"FAIL: {e} in {file}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)

    old_n = len(old_block.splitlines())
    new_n = len(new_block.splitlines())

    if brief:
        if dry_run:
            print(f"OK: would replace {old_n} lines with {new_n} lines in {file} (dry-run)")
        else:
            print(f"OK: replaced {old_n} lines with {new_n} lines in {file}")
        return

    if dry_run:
        console.print("[yellow]DRY RUN: file was not modified.[/yellow]")
        console.print(f"Would replace {old_n} lines with {new_n} lines.")
        diff_text = diff_preview(old_block, new_block, filepath=file)
        if diff_text:
            console.print()
            console.print(Syntax(diff_text, "diff", theme="monokai"))
    else:
        console.print(f"[green]Successfully replaced block in {file}[/green]")


@app.command("outline")
def outline(
    file: Path = typer.Argument(..., help="Target file"),
    imports: bool = typer.Option(False, "--imports", help="Include import statements"),
    docstrings: bool = typer.Option(False, "--docstrings", help="Include first line of docstrings"),
    output: Optional[Path] = typer.Option(None, "--output", help="Output file path"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Extract function/class signatures from a file (bodies omitted)."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    result = outline_file(file, include_imports=imports, include_docstrings=docstrings)
    
    if brief:
        print(f"OK: {len(result)} symbols in {file}")
        return

    if not result:
        console.print("[yellow]No symbols found.[/yellow]")
        return
        
    out_text = "\n".join(result) + "\n"
    if output:
        output.write_text(out_text, encoding="utf-8")
        console.print(f"[green]Extracted outline to {output}[/green]")
    else:
        print(out_text, end="")


@app.command("context")
def context(
    file: Path = typer.Argument(..., help="Target file"),
    function: Optional[str] = typer.Option(None, "--function", help="Function name"),
    symbol: Optional[str] = typer.Option(None, "--symbol", help="Symbol name (alias for --function)"),
    margin: int = typer.Option(5, "--margin", help="Number of lines to include before and after the symbol"),
    output: Optional[Path] = typer.Option(None, "--output", help="Output file path"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Extract a symbol with surrounding context lines, numbered."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    effective_function = symbol or function
    if not effective_function:
        console.print("[red]Error:[/red] Provide --function or --symbol.")
        raise typer.Exit(1)
    if symbol and function:
        console.print("[red]Error:[/red] Use either --function or --symbol, not both.")
        raise typer.Exit(1)

    try:
        content = extract_context(file, effective_function, margin=margin)
    except Exception as e:
        if brief:
            print(f"FAIL: {e} in {file}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)
        
    if brief:
        lines_count = len(content.splitlines()) - 1  # -1 for header
        print(f"OK: extracted {lines_count} lines of context from {file}")
        return

    if output:
        output.write_text(content, encoding="utf-8")
        console.print(f"[green]Extracted context to {output}[/green]")
    else:
        print(content, end="")
