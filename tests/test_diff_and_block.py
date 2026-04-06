"""Tests for diff summarize command and core logic."""

from pathlib import Path

import pytest
from typer.testing import CliRunner

from devkit.commands import diff as diff_commands
from devkit.core import diff as diff_core

runner = CliRunner()


def test_summarize_diff_includes_scope_for_range(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(diff_core, "run_git_command", lambda args: "5\t2\tsrc/app.py")

    summary = diff_core.summarize_diff(base="origin/main", head="HEAD")

    assert summary["scope"] == {
        "mode": "range",
        "description": "range origin/main...HEAD",
        "refspec": "origin/main...HEAD",
    }
    assert summary["files"][0]["path"] == "src/app.py"


def test_diff_summarize_prints_scope_when_no_changes(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        diff_commands,
        "summarize_diff",
        lambda **kwargs: {
            "scope": {"mode": "range", "description": "range origin/main...HEAD", "refspec": "origin/main...HEAD"},
            "files": [],
            "total_additions": 0,
            "total_deletions": 0,
        },
    )

    result = runner.invoke(diff_commands.app, ["--base", "origin/main", "--head", "HEAD"])

    assert result.exit_code == 0
    assert "No changes found for range origin/main...HEAD." in result.stdout


def test_diff_summarize_reports_scope_validation_errors() -> None:
    result = runner.invoke(diff_commands.app, ["--staged", "--base", "main", "--head", "HEAD"])

    assert result.exit_code == 1
    assert "cannot be combined" in result.stdout
