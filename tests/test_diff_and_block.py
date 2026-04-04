from pathlib import Path

import pytest
from typer.testing import CliRunner

from devkit.commands import block as block_commands
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


def test_block_extract_lists_headings(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\n\n## Details\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--list-headings"])

    assert result.exit_code == 0
    assert "L1: # Title" in result.stdout
    assert "L3: ## Details" in result.stdout


def test_block_extract_lists_functions(tmp_path: Path) -> None:
    target = tmp_path / "sample.py"
    target.write_text("def first():\n    pass\n\nclass Second:\n    pass\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--list-functions"])

    assert result.exit_code == 0
    assert "L1: first" in result.stdout
    assert "L4: Second" in result.stdout


def test_block_extract_suggests_close_heading(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Intro\n\n## Install Guide\ncontent\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--heading", "Install Gude"])

    assert result.exit_code == 1
    assert "Did you mean" in result.stdout
    assert "## Install Guide" in result.stdout
