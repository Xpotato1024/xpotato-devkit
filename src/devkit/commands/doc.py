"""CLI commands for document template generation."""

import typer
from pathlib import Path
from typing import Optional
from rich.console import Console

from devkit.core.config import load_config
from devkit.core.doc import generate_benchmark_note, generate_impl_note

app = typer.Typer()
console = Console()


def _try_get_summary(
    staged: bool, base: Optional[str], head: Optional[str], commits: Optional[str]
):
    """Attempt to get a diff summary; return None on failure."""
    if not any([staged, base, head, commits]):
        return None
    try:
        from devkit.core.diff import summarize_diff
        return summarize_diff(staged=staged, base=base, head=head, commits=commits)
    except Exception:
        return None


@app.command("impl-note")
def impl_note(
    staged: bool = typer.Option(False, "--staged", help="Use staged diff for context"),
    base: Optional[str] = typer.Option(None, "--base", help="Base ref for diff context"),
    head: Optional[str] = typer.Option(None, "--head", help="Head ref for diff context"),
    commits: Optional[str] = typer.Option(None, "--commits", help="Git revision range for diff context"),
    lang: Optional[str] = typer.Option(None, "--lang", help="Language for the template (ja/en)"),
    output: Optional[Path] = typer.Option(None, "--output", help="File to write the template to"),
):
    """Generate an implementation note template."""
    config = load_config()
    lang_to_use = lang or config.get("git", {}).get("lang", "ja")
    summary = _try_get_summary(staged, base, head, commits)
    content = generate_impl_note(summary=summary, lang=lang_to_use)

    if output:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding="utf-8")
        console.print(f"[green]Implementation note template written to {output}[/green]")
    else:
        print(content, end="")


@app.command("benchmark-note")
def benchmark_note(
    staged: bool = typer.Option(False, "--staged", help="Use staged diff for context"),
    base: Optional[str] = typer.Option(None, "--base", help="Base ref for diff context"),
    head: Optional[str] = typer.Option(None, "--head", help="Head ref for diff context"),
    commits: Optional[str] = typer.Option(None, "--commits", help="Git revision range for diff context"),
    lang: Optional[str] = typer.Option(None, "--lang", help="Language for the template (ja/en)"),
    output: Optional[Path] = typer.Option(None, "--output", help="File to write the template to"),
):
    """Generate a benchmark note template."""
    config = load_config()
    lang_to_use = lang or config.get("git", {}).get("lang", "ja")
    summary = _try_get_summary(staged, base, head, commits)
    content = generate_benchmark_note(summary=summary, lang=lang_to_use)

    if output:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding="utf-8")
        console.print(f"[green]Benchmark note template written to {output}[/green]")
    else:
        print(content, end="")
