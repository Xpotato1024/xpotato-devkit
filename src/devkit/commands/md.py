"""CLI commands for Markdown section manipulation."""

import typer
from pathlib import Path
from typing import Optional
from rich.console import Console

from devkit.core.md import append_bullet, append_to_section, ensure_section, replace_section

app = typer.Typer()
console = Console()


@app.command("append-section")
def cmd_append_section(
    file: Path = typer.Argument(..., help="Target Markdown file"),
    heading: str = typer.Option(..., "--heading", help="Heading text to append to"),
    content: str = typer.Option(None, "--content", help="Content to append (inline)"),
    content_file: Optional[Path] = typer.Option(None, "--content-file", help="File containing content to append"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Preview without modifying the file"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Append content to the end of a Markdown section."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    text = _resolve_content(content, content_file)
    if text is None:
        console.print("[red]Error:[/red] Provide either --content or --content-file.")
        raise typer.Exit(1)

    try:
        append_to_section(file, heading, text, dry_run=dry_run)
    except ValueError as e:
        if brief:
            print(f"FAIL: {e}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)

    if brief:
        suffix = " (dry-run)" if dry_run else ""
        print(f"OK: appended to '{heading}' in {file}{suffix}")
    elif dry_run:
        console.print("[yellow]DRY RUN: file was not modified.[/yellow]")
    else:
        console.print(f"[green]Appended to section '{heading}' in {file}[/green]")


@app.command("replace-section")
def cmd_replace_section(
    file: Path = typer.Argument(..., help="Target Markdown file"),
    heading: str = typer.Option(..., "--heading", help="Heading text to replace"),
    content: str = typer.Option(None, "--content", help="Replacement content (inline)"),
    content_file: Optional[Path] = typer.Option(None, "--content-file", help="File containing replacement content"),
    keep_heading: bool = typer.Option(True, "--keep-heading/--no-keep-heading", help="Keep the heading line itself"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Preview without modifying the file"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Replace the body of a Markdown section."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    text = _resolve_content(content, content_file)
    if text is None:
        console.print("[red]Error:[/red] Provide either --content or --content-file.")
        raise typer.Exit(1)

    try:
        replace_section(file, heading, text, keep_heading=keep_heading, dry_run=dry_run)
    except ValueError as e:
        if brief:
            print(f"FAIL: {e}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)

    if brief:
        suffix = " (dry-run)" if dry_run else ""
        print(f"OK: replaced section '{heading}' in {file}{suffix}")
    elif dry_run:
        console.print("[yellow]DRY RUN: file was not modified.[/yellow]")
    else:
        console.print(f"[green]Replaced section '{heading}' in {file}[/green]")


@app.command("ensure-section")
def cmd_ensure_section(
    file: Path = typer.Argument(..., help="Target Markdown file"),
    heading: str = typer.Option(..., "--heading", help="Heading text to ensure exists"),
    content: str = typer.Option("", "--content", help="Initial content if section is created"),
    content_file: Optional[Path] = typer.Option(None, "--content-file", help="File containing initial content"),
    level: int = typer.Option(2, "--level", help="Heading level (number of #) if created"),
    after: Optional[str] = typer.Option(None, "--after", help="Insert after this heading if section doesn't exist"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Preview without modifying the file"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Ensure a Markdown section exists; create it if missing."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    text = _resolve_content(content, content_file) or ""

    try:
        ensure_section(file, heading, text, level=level, after=after, dry_run=dry_run)
    except ValueError as e:
        if brief:
            print(f"FAIL: {e}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)

    if brief:
        suffix = " (dry-run)" if dry_run else ""
        print(f"OK: section '{heading}' ensured in {file}{suffix}")
    elif dry_run:
        console.print("[yellow]DRY RUN: file was not modified.[/yellow]")
    else:
        console.print(f"[green]Section '{heading}' ensured in {file}[/green]")


@app.command("append-bullet")
def cmd_append_bullet(
    file: Path = typer.Argument(..., help="Target Markdown file"),
    heading: str = typer.Option(..., "--heading", help="Heading text of the target section"),
    bullet: str = typer.Option(..., "--bullet", help="Bullet text to append (without leading '- ')"),
    dedupe: bool = typer.Option(False, "--dedupe", help="Skip if an identical bullet already exists"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Preview without modifying the file"),
    brief: bool = typer.Option(False, "--brief", help="Output a single summary line"),
):
    """Append a bullet item to a Markdown section."""
    if not file.is_file():
        console.print(f"[red]Error:[/red] File {file} does not exist.")
        raise typer.Exit(1)

    try:
        append_bullet(file, heading, bullet, dedupe=dedupe, dry_run=dry_run)
    except ValueError as e:
        if brief:
            print(f"FAIL: {e}")
        else:
            console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)

    if brief:
        suffix = " (dry-run)" if dry_run else ""
        print(f"OK: bullet appended to '{heading}' in {file}{suffix}")
    elif dry_run:
        console.print("[yellow]DRY RUN: file was not modified.[/yellow]")
    else:
        console.print(f"[green]Bullet appended to '{heading}' in {file}[/green]")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _resolve_content(content: Optional[str], content_file: Optional[Path]) -> Optional[str]:
    """Return content from either inline text or file."""
    if content_file and content_file.is_file():
        return content_file.read_text(encoding="utf-8")
    if content:
        return content
    return None
