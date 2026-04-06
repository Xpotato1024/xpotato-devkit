import typer

from devkit.commands.bootstrap import app as bootstrap_app
from devkit.commands.block import app as block_app
from devkit.commands.diff import app as diff_app
from devkit.commands.doc import app as doc_app
from devkit.commands.encoding import app as encoding_app
from devkit.commands.git import app as git_app
from devkit.commands.md import app as md_app
from devkit.commands.patch import app as patch_app
from devkit.commands.tree import app as tree_app
from devkit.commands.metrics import app as metrics_app

from devkit.core.timing import TimingContext, emit_timing, set_context

app = typer.Typer(help="Repo-agnostic CLI utilities for AI-assisted development.")


@app.callback()
def main_callback(
    ctx: typer.Context,
    time_flag: bool = typer.Option(False, "--time", help="Print execution timing to stderr"),
    time_json: bool = typer.Option(False, "--time-json", help="Print execution timing as JSON to stderr"),
):
    """Root callback for global options."""
    if time_flag or time_json:
        timing_ctx = TimingContext()
        timing_ctx.start()
        set_context(timing_ctx)
        ctx.call_on_close(lambda: emit_timing("json" if time_json else "human"))
    else:
        set_context(None)


app.add_typer(encoding_app, name="encoding")
app.add_typer(diff_app, name="diff")
app.add_typer(block_app, name="block")
app.add_typer(patch_app, name="patch")
app.add_typer(git_app, name="git")
app.add_typer(md_app, name="md")
app.add_typer(doc_app, name="doc")
app.add_typer(tree_app, name="tree")
app.add_typer(bootstrap_app, name="bootstrap")
app.add_typer(metrics_app, name="metrics")

if __name__ == "__main__":
    import sys
    import time
    from devkit.core.metrics import record_metric

    start = time.perf_counter()
    ok = True
    try:
        app()
    except SystemExit as e:
        ok = (e.code == 0 or e.code is None)
        raise
    except Exception:
        ok = False
        raise
    finally:
        end = time.perf_counter()
        dur = (end - start) * 1000
        # Determine command heuristically
        args = [a for a in sys.argv[1:] if not a.startswith("-")]
        cmd = " ".join(args[:2]) if args else "unknown"
        brief = "--brief" in sys.argv
        record_metric(cmd, dur, brief, ok)
