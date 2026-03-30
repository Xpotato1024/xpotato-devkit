import typer
from pathlib import Path
from typing import Optional
from rich.console import Console
import subprocess

from devkit.core.git import (
    generate_commit_template, 
    generate_pr_template, 
    check_safe_branch,
    check_upstream,
    get_current_branch
)
from devkit.core.config import load_config

app = typer.Typer()
console = Console()

@app.command("commit-message")
def commit_message(
    staged: bool = typer.Option(False, "--staged", help="Use staged diff instead of unstaged"),
    lang: Optional[str] = typer.Option(None, "--lang", help="Language for the instructions (ja/en)"),
    output: Optional[Path] = typer.Option(None, "--output", help="File to write the draft to"),
):
    """Generate a commit message draft/template with AI instructions."""
    config = load_config()
    lang_to_use = lang or config.get("git", {}).get("lang", "ja")
    
    try:
        content = generate_commit_template(staged=staged, lang=lang_to_use)
    except Exception as e:
        console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)
        
    if output:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding="utf-8")
        console.print(f"[green]Commit message template written to {output}[/green]")
    else:
        print(content)

@app.command("pr-body")
def pr_body(
    base: str = typer.Option("main", "--base", help="Base branch to compare against"),
    lang: Optional[str] = typer.Option(None, "--lang", help="Language for the PR template (ja/en)"),
    output: Optional[Path] = typer.Option(None, "--output", help="File to write the PR body to"),
):
    """Generate a PR body template."""
    config = load_config()
    lang_to_use = lang or config.get("git", {}).get("lang", "ja")

    try:
        content = generate_pr_template(base=base, lang=lang_to_use)
    except Exception as e:
        console.print(f"[red]Error:[/red] {e}")
        raise typer.Exit(1)
        
    if output:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding="utf-8")
        console.print(f"[green]PR body template written to {output}[/green]")
    else:
        print(content)

@app.command("safe-push")
def safe_push(
    yes: bool = typer.Option(False, "--yes", "-y", help="Do not prompt for confirmation"),
    no_confirm: bool = typer.Option(False, "--no-confirm", help="Alias for --yes"),
):
    """Safely push the current branch to upstream, preventing direct pushes to main/master."""
    try:
        check_safe_branch()
    except RuntimeError as e:
        console.print(f"[red]Safety Check Failed:[/red] {e}")
        raise typer.Exit(1)
        
    current = get_current_branch()
    console.print(f"Pushing branch [bold cyan]{current}[/bold cyan]...")
    
    args = ["git", "push"]
    if not check_upstream():
        console.print("[yellow]No upstream set. Will automatically set upstream.[/yellow]")
        args.extend(["-u", "origin", current])
        
    do_push = yes or no_confirm
    if not do_push:
        do_push = typer.confirm("Are you sure you want to push?")
        
    if do_push:
        result = subprocess.run(args)
        if result.returncode != 0:
            console.print("[red]Push failed.[/red]")
            raise typer.Exit(1)
        else:
            console.print("[green]Successfully pushed.[/green]")
    else:
        console.print("[yellow]Aborted.[/yellow]")
