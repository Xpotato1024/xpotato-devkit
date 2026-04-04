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
    get_current_branch,
    get_upstream_remote,
)
from devkit.core.config import load_config

app = typer.Typer()
console = Console()

@app.command("commit-message")
def commit_message(
    staged: bool = typer.Option(False, "--staged", help="Use staged diff instead of unstaged"),
    base: Optional[str] = typer.Option(None, "--base", help="Base ref for an explicit diff range"),
    head: Optional[str] = typer.Option(None, "--head", help="Head ref for an explicit diff range"),
    commits: Optional[str] = typer.Option(None, "--commits", help="Git revision range such as A..B or A...B"),
    lang: Optional[str] = typer.Option(None, "--lang", help="Language for the instructions (ja/en)"),
    output: Optional[Path] = typer.Option(None, "--output", help="File to write the draft to"),
):
    """Generate a commit message draft/template with AI instructions."""
    try:
        config = load_config()
        lang_to_use = lang or config.get("git", {}).get("lang", "ja")
        content = generate_commit_template(staged=staged, base=base, head=head, commits=commits, lang=lang_to_use)
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
    staged: bool = typer.Option(False, "--staged", help="Use staged diff instead of unstaged"),
    base: Optional[str] = typer.Option(None, "--base", help="Base ref for an explicit diff range"),
    head: Optional[str] = typer.Option(None, "--head", help="Head ref for an explicit diff range"),
    commits: Optional[str] = typer.Option(None, "--commits", help="Git revision range such as A..B or A...B"),
    lang: Optional[str] = typer.Option(None, "--lang", help="Language for the PR template (ja/en)"),
    output: Optional[Path] = typer.Option(None, "--output", help="File to write the PR body to"),
):
    """Generate a PR body template."""
    try:
        config = load_config()
        lang_to_use = lang or config.get("git", {}).get("lang", "ja")
        content = generate_pr_template(staged=staged, base=base, head=head, commits=commits, lang=lang_to_use)
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
    remote: Optional[str] = typer.Option(None, "--remote", help="Remote name to use when setting upstream if none is configured"),
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
    tracking_set = False
    target_remote = None
    if not check_upstream():
        if not remote:
            console.print("[red]No upstream is configured for this branch.[/red] Use --remote to set one or configure tracking first.")
            raise typer.Exit(1)
        console.print(f"[yellow]No upstream set. Will automatically set upstream to {remote}.[/yellow]")
        args.extend(["-u", remote, current])
        tracking_set = True
        target_remote = remote
    else:
        target_remote = get_upstream_remote()

    do_push = yes or no_confirm
    if not do_push:
        do_push = typer.confirm("Are you sure you want to push?")

    if do_push:
        console.print(f"Running [bold]{' '.join(args)}[/bold]")
        result = subprocess.run(args, check=False)
        if result.returncode != 0:
            console.print("[red]Push failed.[/red]")
            raise typer.Exit(1)
        else:
            tracking_message = "set" if tracking_set else "unchanged"
            console.print(
                f"[green]Successfully pushed branch {current} to {target_remote}. Tracking: {tracking_message}.[/green]"
            )
    else:
        console.print("[yellow]Aborted.[/yellow]")
