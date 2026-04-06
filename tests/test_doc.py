"""Tests for document template generation (devkit doc)."""

from typer.testing import CliRunner

from devkit.commands import doc as doc_commands
from devkit.core import doc as doc_core

runner = CliRunner()


# ---------------------------------------------------------------------------
# Core template generation
# ---------------------------------------------------------------------------

def test_impl_note_ja_basic() -> None:
    content = doc_core.generate_impl_note(lang="ja")
    assert "## 変更概要" in content
    assert "## 背景" in content
    assert "## 実装内容" in content
    assert "## 検証" in content


def test_impl_note_en_basic() -> None:
    content = doc_core.generate_impl_note(lang="en")
    assert "## Summary" in content
    assert "## Background" in content
    assert "## Changes" in content
    assert "## Verification" in content


def test_impl_note_with_summary() -> None:
    summary = {
        "scope": {"mode": "staged", "description": "staged changes", "refspec": None},
        "files": [
            {"path": "src/app.py", "additions": 10, "deletions": 3, "is_binary": False},
            {"path": "docs/readme.md", "additions": 5, "deletions": 0, "is_binary": False},
        ],
        "total_additions": 15,
        "total_deletions": 3,
    }
    content = doc_core.generate_impl_note(summary=summary, lang="ja")
    assert "`src/app.py`" in content
    assert "`docs/readme.md`" in content
    assert "+15/-3" in content


def test_benchmark_note_ja() -> None:
    content = doc_core.generate_benchmark_note(lang="ja")
    assert "## ベンチマーク概要" in content
    assert "## 環境" in content
    assert "## 結果" in content
    assert "Before" in content


def test_benchmark_note_en() -> None:
    content = doc_core.generate_benchmark_note(lang="en")
    assert "## Benchmark Summary" in content
    assert "## Environment" in content
    assert "## Results" in content


# ---------------------------------------------------------------------------
# CLI integration
# ---------------------------------------------------------------------------

def test_doc_impl_note_cli() -> None:
    result = runner.invoke(doc_commands.app, ["impl-note", "--lang", "en"])
    assert result.exit_code == 0
    assert "## Summary" in result.stdout


def test_doc_benchmark_note_cli() -> None:
    result = runner.invoke(doc_commands.app, ["benchmark-note", "--lang", "ja"])
    assert result.exit_code == 0
    assert "ベンチマーク" in result.stdout
