"""Tests for --brief mode, --files-only, block outline, block context, and devkit tree."""

from pathlib import Path
from textwrap import dedent

import pytest
from typer.testing import CliRunner

from devkit.commands import block as block_commands
from devkit.commands import diff as diff_commands
from devkit.commands import encoding as encoding_commands
from devkit.commands import md as md_commands
from devkit.commands import patch as patch_commands
from devkit.commands import tree as tree_commands
from devkit.core import block as block_core
from devkit.core import diff as diff_core
from devkit.core import tree as tree_core

runner = CliRunner()


# ===========================================================================
# --brief mode tests
# ===========================================================================

def test_encoding_check_brief_ok(tmp_path: Path) -> None:
    f = tmp_path / "clean.txt"
    f.write_text("hello\n", encoding="utf-8")

    result = runner.invoke(encoding_commands.app, ["check", str(f), "--brief"])

    assert result.exit_code == 0
    assert result.stdout.startswith("OK:")
    assert "1 files checked" in result.stdout


def test_encoding_check_brief_fail(tmp_path: Path) -> None:
    f = tmp_path / "bom.txt"
    f.write_bytes(b"\xef\xbb\xbfhello\n")

    result = runner.invoke(encoding_commands.app, ["check", str(f), "--brief"])

    assert result.exit_code == 1
    assert result.stdout.startswith("FAIL:")
    assert "BOM" in result.stdout


def test_diff_summarize_brief(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        diff_commands,
        "summarize_diff",
        lambda **kw: {
            "scope": {"mode": "staged", "description": "staged", "refspec": None},
            "files": [{"path": "a.py", "additions": 10, "deletions": 3, "is_binary": False}],
            "total_additions": 10,
            "total_deletions": 3,
        },
    )

    result = runner.invoke(diff_commands.app, ["--staged", "--brief"])

    assert result.exit_code == 0
    assert "OK:" in result.stdout
    assert "+10/-3" in result.stdout


def test_diff_summarize_files_only(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        diff_commands,
        "summarize_diff",
        lambda **kw: {
            "scope": {"mode": "staged", "description": "staged", "refspec": None},
            "files": [
                {"path": "a.py", "additions": 5, "deletions": 0, "is_binary": False},
                {"path": "b.py", "additions": 3, "deletions": 1, "is_binary": False},
            ],
            "total_additions": 8,
            "total_deletions": 1,
        },
    )

    result = runner.invoke(diff_commands.app, ["--staged", "--files-only"])

    assert result.exit_code == 0
    assert "a.py\n" in result.stdout
    assert "b.py\n" in result.stdout
    # Should NOT contain table headers
    assert "Additions" not in result.stdout


def test_block_extract_brief(tmp_path: Path) -> None:
    f = tmp_path / "sample.py"
    f.write_text("def hello():\n    return 42\n\ndef other():\n    pass\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(f), "--function", "hello", "--brief"])

    assert result.exit_code == 0
    assert result.stdout.startswith("OK:")
    assert "extracted" in result.stdout


def test_block_replace_brief(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\nold\n\n## Other\nstuff\n", encoding="utf-8")
    replacement = tmp_path / "new.md"
    replacement.write_text("# Title\nnew content\n", encoding="utf-8")

    result = runner.invoke(
        block_commands.app,
        ["replace", str(target), "--with-file", str(replacement), "--heading", "# Title", "--brief"],
    )

    assert result.exit_code == 0
    assert result.stdout.startswith("OK:")
    assert "replaced" in result.stdout


def test_md_append_section_brief(tmp_path: Path) -> None:
    f = tmp_path / "doc.md"
    f.write_text("# Title\n\n## Log\n- item 1\n", encoding="utf-8")

    result = runner.invoke(
        md_commands.app,
        ["append-section", str(f), "--heading", "Log", "--content", "- item 2", "--brief"],
    )

    assert result.exit_code == 0
    assert result.stdout.startswith("OK:")
    assert "Log" in result.stdout


# ===========================================================================
# block outline tests
# ===========================================================================

def test_outline_python(tmp_path: Path) -> None:
    f = tmp_path / "sample.py"
    f.write_text(dedent("""\
        import os
        from pathlib import Path

        def greet(name: str) -> str:
            \"\"\"Say hello.\"\"\"
            return f"Hello, {name}"

        class MyClass:
            def method(self):
                pass
    """), encoding="utf-8")

    result = block_core.outline_file(f)

    assert any("def greet" in line for line in result)
    assert any("class MyClass" in line for line in result)
    assert any("def method" in line for line in result)
    # Should NOT include function bodies
    assert not any("return f" in line for line in result)


def test_outline_with_imports(tmp_path: Path) -> None:
    f = tmp_path / "sample.py"
    f.write_text("import os\nfrom pathlib import Path\n\ndef hello(): pass\n", encoding="utf-8")

    result = block_core.outline_file(f, include_imports=True)

    assert any("import os" in line for line in result)
    assert any("from pathlib" in line for line in result)


def test_outline_cli(tmp_path: Path) -> None:
    f = tmp_path / "sample.py"
    f.write_text("def foo(x: int) -> int:\n    return x\n\ndef bar():\n    pass\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["outline", str(f)])

    assert result.exit_code == 0
    assert "def foo" in result.stdout
    assert "def bar" in result.stdout
    assert "return x" not in result.stdout


def test_outline_with_decorators(tmp_path: Path) -> None:
    f = tmp_path / "sample.py"
    f.write_text(dedent("""\
        @app.command("run")
        def run_cmd():
            pass
    """), encoding="utf-8")

    result = block_core.outline_file(f)

    assert any("@app.command" in line for line in result)
    assert any("def run_cmd" in line for line in result)


# ===========================================================================
# block context tests
# ===========================================================================

def test_context_basic(tmp_path: Path) -> None:
    f = tmp_path / "sample.py"
    f.write_text(dedent("""\
        import os

        MAX = 10

        def greet(name):
            return f"Hello, {name}"

        def farewell(name):
            return f"Bye, {name}"

        def other():
            pass
    """), encoding="utf-8")

    content = block_core.extract_context(f, "greet", margin=2)

    assert "greet" in content
    assert "MAX" in content  # margin includes preceding lines
    # Line numbers should be present
    assert "5:" in content or "4:" in content


def test_context_cli(tmp_path: Path) -> None:
    f = tmp_path / "sample.py"
    f.write_text("def hello():\n    return 42\n\ndef world():\n    return 0\n", encoding="utf-8")

    result = runner.invoke(
        block_commands.app,
        ["context", str(f), "--symbol", "hello", "--margin", "1"],
    )

    assert result.exit_code == 0
    assert "hello" in result.stdout


# ===========================================================================
# devkit tree tests
# ===========================================================================

def test_tree_basic(tmp_path: Path) -> None:
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "main.py").write_text("pass\n", encoding="utf-8")
    (tmp_path / "README.md").write_text("# Hi\n", encoding="utf-8")

    entry = tree_core.scan_tree(tmp_path, use_gitignore=False)
    lines = tree_core.format_tree(entry)

    text = "\n".join(lines)
    assert "src/" in text
    assert "main.py" in text
    assert "README.md" in text


def test_tree_respects_gitignore(tmp_path: Path) -> None:
    (tmp_path / ".gitignore").write_text("*.pyc\n__pycache__/\n", encoding="utf-8")
    (tmp_path / "app.py").write_text("pass\n", encoding="utf-8")
    (tmp_path / "__pycache__").mkdir()
    (tmp_path / "__pycache__" / "app.cpython-312.pyc").write_bytes(b"cache")

    entry = tree_core.scan_tree(tmp_path, use_gitignore=True)
    lines = tree_core.format_tree(entry)

    text = "\n".join(lines)
    assert "app.py" in text
    assert "__pycache__" not in text
    assert ".pyc" not in text


def test_tree_max_depth(tmp_path: Path) -> None:
    deep = tmp_path / "a" / "b" / "c"
    deep.mkdir(parents=True)
    (deep / "deep.txt").write_text("x\n", encoding="utf-8")

    entry = tree_core.scan_tree(tmp_path, max_depth=1, use_gitignore=False)
    lines = tree_core.format_tree(entry)

    text = "\n".join(lines)
    assert "a/" in text
    # b/ should appear but c/ should not (depth limited)
    assert "deep.txt" not in text


def test_tree_ext_filter(tmp_path: Path) -> None:
    (tmp_path / "app.py").write_text("pass\n", encoding="utf-8")
    (tmp_path / "data.json").write_text("{}\n", encoding="utf-8")
    (tmp_path / "style.css").write_text("body{}\n", encoding="utf-8")

    entry = tree_core.scan_tree(tmp_path, extensions={".py"}, use_gitignore=False)
    lines = tree_core.format_tree(entry)

    text = "\n".join(lines)
    assert "app.py" in text
    assert "data.json" not in text
    assert "style.css" not in text


def test_tree_summary(tmp_path: Path) -> None:
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "a.py").write_text("x\n", encoding="utf-8")
    (tmp_path / "src" / "b.py").write_text("y\n", encoding="utf-8")

    entry = tree_core.scan_tree(tmp_path, use_gitignore=False)
    summary = tree_core.tree_summary(entry)

    assert "1 directories" in summary
    assert "2 files" in summary


def test_tree_cli(tmp_path: Path) -> None:
    (tmp_path / "hello.py").write_text("pass\n", encoding="utf-8")

    result = runner.invoke(tree_commands.app, ["--path", str(tmp_path), "--no-gitignore"])

    assert result.exit_code == 0
    assert "hello.py" in result.stdout
