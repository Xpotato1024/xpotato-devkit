import typer

from devkit.commands.block import app as block_app
from devkit.commands.diff import app as diff_app
from devkit.commands.encoding import app as encoding_app
from devkit.commands.git import app as git_app
from devkit.commands.patch import app as patch_app

app = typer.Typer(help="Repo-agnostic CLI utilities for AI-assisted development.")

app.add_typer(encoding_app, name="encoding")
app.add_typer(diff_app, name="diff")
app.add_typer(block_app, name="block")
app.add_typer(patch_app, name="patch")
app.add_typer(git_app, name="git")

if __name__ == "__main__":
    app()
