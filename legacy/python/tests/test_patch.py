"""Tests for patch module enhancements."""

from pathlib import Path
from textwrap import dedent

import pytest
from typer.testing import CliRunner

from devkit.commands import patch as patch_commands
from devkit.core import patch as patch_core

runner = CliRunner()


# ---------------------------------------------------------------------------
# Patch parsing
# ---------------------------------------------------------------------------

def test_parse_patch_hunks_basic() -> None:
    patch_text = dedent("""\
        diff --git a/src/app.py b/src/app.py
        --- a/src/app.py
        +++ b/src/app.py
        @@ -10,3 +10,4 @@ def main():
             pass
        +    print("hello")
             return
        @@ -20,2 +21,3 @@ def other():
             pass
        +    return 42
    """)

    hunks, files = patch_core.parse_patch_hunks(patch_text)

    assert len(hunks) == 2
    assert files == ["src/app.py"]
    assert hunks[0].old_start == 10
    assert hunks[0].old_count == 3
    assert hunks[0].new_start == 10
    assert hunks[0].new_count == 4
    assert hunks[1].old_start == 20


def test_parse_patch_hunks_multiple_files() -> None:
    patch_text = dedent("""\
        diff --git a/foo.py b/foo.py
        --- a/foo.py
        +++ b/foo.py
        @@ -1,2 +1,3 @@
         line1
        +line2
        diff --git a/bar.py b/bar.py
        --- a/bar.py
        +++ b/bar.py
        @@ -5,1 +5,2 @@
         x
        +y
    """)

    hunks, files = patch_core.parse_patch_hunks(patch_text)

    assert len(hunks) == 2
    assert "foo.py" in files
    assert "bar.py" in files
    assert hunks[0].file == "foo.py"
    assert hunks[1].file == "bar.py"


def test_parse_empty_patch() -> None:
    hunks, files = patch_core.parse_patch_hunks("")

    assert hunks == []
    assert files == []


# ---------------------------------------------------------------------------
# PatchDiagnostic
# ---------------------------------------------------------------------------

def test_diagnostic_summary_success() -> None:
    diag = patch_core.PatchDiagnostic(
        success=True,
        total_hunks=3,
        applied_hunks=3,
        affected_files=["a.py", "b.py"],
    )

    summary = diag.summary()
    assert "applied cleanly" in summary
    assert "3 hunk" in summary


def test_diagnostic_summary_failure() -> None:
    diag = patch_core.PatchDiagnostic(
        success=False,
        total_hunks=5,
        failed_hunks=2,
        applied_hunks=3,
        errors=["error at hunk 1", "error at hunk 3"],
        affected_files=["main.py"],
    )

    summary = diag.summary()
    assert "FAILED" in summary
    assert "2/5" in summary
    assert "main.py" in summary


# ---------------------------------------------------------------------------
# CLI smoke tests
# ---------------------------------------------------------------------------

def test_patch_apply_missing_file() -> None:
    result = runner.invoke(patch_commands.app, ["apply", "--patch-file", "nonexistent.patch"])
    assert result.exit_code == 1
    assert "does not exist" in result.stdout


def test_patch_diagnose_missing_file() -> None:
    result = runner.invoke(patch_commands.app, ["diagnose", "--patch-file", "nonexistent.patch"])
    assert result.exit_code == 1
    assert "does not exist" in result.stdout
