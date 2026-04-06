"""Tests for Markdown section manipulation (devkit md)."""

from pathlib import Path
from textwrap import dedent

import pytest
from typer.testing import CliRunner

from devkit.commands import md as md_commands
from devkit.core import md as md_core

runner = CliRunner()


# ---------------------------------------------------------------------------
# append_to_section
# ---------------------------------------------------------------------------

def test_append_to_section_basic(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text(dedent("""\
        # Title
        intro

        ## Details
        some details

        ## Other
        other stuff
    """), encoding="utf-8")

    md_core.append_to_section(target, "Details", "appended line\n")

    content = target.read_text(encoding="utf-8")
    # "appended line" should appear before "## Other"
    assert "appended line" in content
    append_pos = content.index("appended line")
    other_pos = content.index("## Other")
    assert append_pos < other_pos


def test_append_to_section_cli(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\nintro\n\n## Log\n- entry 1\n", encoding="utf-8")

    result = runner.invoke(
        md_commands.app,
        ["append-section", str(target), "--heading", "Log", "--content", "- entry 2"],
    )

    assert result.exit_code == 0
    content = target.read_text(encoding="utf-8")
    assert "- entry 1" in content
    assert "- entry 2" in content


# ---------------------------------------------------------------------------
# replace_section
# ---------------------------------------------------------------------------

def test_replace_section_keeps_heading(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text(dedent("""\
        # Title
        intro

        ## Details
        old content

        ## Other
        stuff
    """), encoding="utf-8")

    md_core.replace_section(target, "Details", "new content\n")

    content = target.read_text(encoding="utf-8")
    assert "## Details" in content
    assert "new content" in content
    assert "old content" not in content
    assert "## Other" in content


def test_replace_section_cli(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\n\n## Section\nold\n", encoding="utf-8")

    result = runner.invoke(
        md_commands.app,
        ["replace-section", str(target), "--heading", "Section", "--content", "replaced"],
    )

    assert result.exit_code == 0
    content = target.read_text(encoding="utf-8")
    assert "replaced" in content
    assert "old" not in content


# ---------------------------------------------------------------------------
# ensure_section
# ---------------------------------------------------------------------------

def test_ensure_section_creates_when_missing(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\nintro\n", encoding="utf-8")

    md_core.ensure_section(target, "New Section", "initial content\n", level=2)

    content = target.read_text(encoding="utf-8")
    assert "## New Section" in content
    assert "initial content" in content


def test_ensure_section_noop_when_exists(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    original = "# Title\nintro\n\n## Existing\nexisting content\n"
    target.write_text(original, encoding="utf-8")

    md_core.ensure_section(target, "Existing", "should not appear\n")

    content = target.read_text(encoding="utf-8")
    assert content == original


def test_ensure_section_after(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text(dedent("""\
        # Title

        ## First
        content 1

        ## Third
        content 3
    """), encoding="utf-8")

    md_core.ensure_section(target, "Second", "content 2\n", level=2, after="First")

    content = target.read_text(encoding="utf-8")
    # "## Second" should appear between "## First" and "## Third"
    first_pos = content.index("## First")
    second_pos = content.index("## Second")
    third_pos = content.index("## Third")
    assert first_pos < second_pos < third_pos


def test_ensure_section_cli(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\nintro\n", encoding="utf-8")

    result = runner.invoke(
        md_commands.app,
        ["ensure-section", str(target), "--heading", "Notes", "--content", "some notes"],
    )

    assert result.exit_code == 0
    assert "## Notes" in target.read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# append_bullet
# ---------------------------------------------------------------------------

def test_append_bullet_basic(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\n\n## Log\n- item 1\n\n## Other\nstuff\n", encoding="utf-8")

    md_core.append_bullet(target, "Log", "item 2")

    content = target.read_text(encoding="utf-8")
    assert "- item 1" in content
    assert "- item 2" in content


def test_append_bullet_dedupe(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\n\n## Log\n- item 1\n\n## Other\n", encoding="utf-8")

    md_core.append_bullet(target, "Log", "item 1", dedupe=True)

    content = target.read_text(encoding="utf-8")
    # Should not duplicate
    assert content.count("- item 1") == 1


def test_append_bullet_cli(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\n\n## Items\n- a\n", encoding="utf-8")

    result = runner.invoke(
        md_commands.app,
        ["append-bullet", str(target), "--heading", "Items", "--bullet", "b"],
    )

    assert result.exit_code == 0
    content = target.read_text(encoding="utf-8")
    assert "- b" in content


# ---------------------------------------------------------------------------
# Frontmatter safety
# ---------------------------------------------------------------------------

def test_frontmatter_preserved_on_replace(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text(dedent("""\
        ---
        title: Test
        date: 2026-01-01
        ---
        # Title
        intro

        ## Section
        old
    """), encoding="utf-8")

    md_core.replace_section(target, "Section", "new content\n")

    content = target.read_text(encoding="utf-8")
    assert content.startswith("---\ntitle: Test")
    assert "new content" in content
    assert "old" not in content


def test_frontmatter_preserved_on_ensure(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text(dedent("""\
        ---
        tags: [a, b]
        ---
        # Title
        intro
    """), encoding="utf-8")

    md_core.ensure_section(target, "Footer", "foot\n")

    content = target.read_text(encoding="utf-8")
    assert content.startswith("---\ntags:")
    assert "## Footer" in content
