"""Tests for block extract / replace and the underlying core logic."""

from pathlib import Path
from textwrap import dedent

import pytest
from typer.testing import CliRunner

from devkit.commands import block as block_commands
from devkit.core import block as block_core

runner = CliRunner()


# ---------------------------------------------------------------------------
# list_markdown_headings
# ---------------------------------------------------------------------------

def test_block_extract_lists_headings(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\n\n## Details\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--list-headings"])

    assert result.exit_code == 0
    assert "L1: # Title" in result.stdout
    assert "L3: ## Details" in result.stdout


def test_list_headings_includes_slug(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Hello World\n\n## Install Guide\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--list-headings"])

    assert result.exit_code == 0
    assert "[hello-world]" in result.stdout
    assert "[install-guide]" in result.stdout


# ---------------------------------------------------------------------------
# list_functions
# ---------------------------------------------------------------------------

def test_block_extract_lists_functions(tmp_path: Path) -> None:
    target = tmp_path / "sample.py"
    target.write_text("def first():\n    pass\n\nclass Second:\n    pass\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--list-functions"])

    assert result.exit_code == 0
    assert "L1: first" in result.stdout
    assert "L4: Second" in result.stdout


def test_list_functions_rust(tmp_path: Path) -> None:
    target = tmp_path / "lib.rs"
    target.write_text(dedent("""\
        pub fn hello() {
            println!("hi");
        }

        struct Point {
            x: f64,
            y: f64,
        }

        impl Point {
            fn new() -> Self {
                Self { x: 0.0, y: 0.0 }
            }
        }
    """), encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--list-functions"])

    assert result.exit_code == 0
    assert "L1: hello" in result.stdout
    assert "L5: Point" in result.stdout
    assert "L10: Point" in result.stdout  # impl Point


def test_list_functions_go(tmp_path: Path) -> None:
    target = tmp_path / "main.go"
    target.write_text(dedent("""\
        func main() {
            fmt.Println("hello")
        }

        func (s *Server) Run() {
            s.start()
        }
    """), encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--list-functions"])

    assert result.exit_code == 0
    assert "L1: main" in result.stdout
    assert "L5: Run" in result.stdout


# ---------------------------------------------------------------------------
# Python indentation-based function extraction
# ---------------------------------------------------------------------------

def test_extract_python_function_with_blank_lines(tmp_path: Path) -> None:
    """Python function with internal blank lines should not be truncated."""
    target = tmp_path / "sample.py"
    target.write_text(dedent("""\
        def greet(name):
            \"\"\"Say hello.\"\"\"

            msg = f"Hello, {name}"

            return msg

        def other():
            pass
    """), encoding="utf-8")

    content = block_core.extract_block(target, function="greet")

    assert "def greet" in content
    assert "return msg" in content
    assert "def other" not in content


def test_extract_python_class_with_methods(tmp_path: Path) -> None:
    target = tmp_path / "sample.py"
    target.write_text(dedent("""\
        class MyClass:
            def __init__(self):
                self.x = 1

            def method(self):
                return self.x

        def standalone():
            pass
    """), encoding="utf-8")

    content = block_core.extract_block(target, function="MyClass")

    assert "class MyClass" in content
    assert "def method" in content
    assert "def standalone" not in content


def test_extract_python_nested_function(tmp_path: Path) -> None:
    target = tmp_path / "sample.py"
    target.write_text(dedent("""\
        def outer():
            def inner():
                pass
            return inner()

        def another():
            pass
    """), encoding="utf-8")

    content = block_core.extract_block(target, function="outer")

    assert "def outer" in content
    assert "def inner" in content
    assert "return inner()" in content
    assert "def another" not in content


# ---------------------------------------------------------------------------
# Rust / Go brace-based function extraction
# ---------------------------------------------------------------------------

def test_extract_rust_function(tmp_path: Path) -> None:
    target = tmp_path / "lib.rs"
    target.write_text(dedent("""\
        pub fn compute(x: i32) -> i32 {
            let y = x * 2;

            if y > 10 {
                return y;
            }
            y + 1
        }

        fn other() {
            println!("other");
        }
    """), encoding="utf-8")

    content = block_core.extract_block(target, function="compute")

    assert "pub fn compute" in content
    assert "y + 1" in content
    assert "fn other" not in content


def test_extract_go_method(tmp_path: Path) -> None:
    target = tmp_path / "server.go"
    target.write_text(dedent("""\
        func (s *Server) Start() {
            s.running = true

            go func() {
                s.listen()
            }()
        }

        func main() {
            s := NewServer()
        }
    """), encoding="utf-8")

    content = block_core.extract_block(target, function="Start")

    assert "func (s *Server) Start()" in content
    assert "s.listen()" in content
    assert "func main()" not in content


# ---------------------------------------------------------------------------
# --symbol option (alias for --function)
# ---------------------------------------------------------------------------

def test_symbol_option_works_like_function(tmp_path: Path) -> None:
    target = tmp_path / "sample.py"
    target.write_text(dedent("""\
        def hello():
            return 42

        def world():
            return 0
    """), encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--symbol", "hello"])

    assert result.exit_code == 0
    assert "def hello" in result.stdout
    assert "def world" not in result.stdout


def test_symbol_and_function_mutual_exclusive(tmp_path: Path) -> None:
    target = tmp_path / "sample.py"
    target.write_text("def x(): pass\n", encoding="utf-8")

    result = runner.invoke(
        block_commands.app,
        ["extract", str(target), "--function", "x", "--symbol", "x"],
    )

    assert result.exit_code == 1
    assert "not both" in result.stdout


# ---------------------------------------------------------------------------
# Heading extraction improvements
# ---------------------------------------------------------------------------

def test_heading_exact_match(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text(dedent("""\
        # Install
        intro

        ## Install Guide
        guide content

        ## Install Notes
        notes content
    """), encoding="utf-8")

    # Partial match: "Install" matches multiple -> disambiguation error
    result = runner.invoke(block_commands.app, ["extract", str(target), "--heading", "Install"])
    assert result.exit_code == 1
    assert "matched" in result.stdout or "disambiguate" in result.stdout

    # Exact match: "Install" matches only the h1
    result2 = runner.invoke(
        block_commands.app,
        ["extract", str(target), "--heading", "Install", "--heading-exact"],
    )
    assert result2.exit_code == 0
    assert "intro" in result2.stdout


def test_block_extract_suggests_close_heading(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Intro\n\n## Install Guide\ncontent\n", encoding="utf-8")

    result = runner.invoke(block_commands.app, ["extract", str(target), "--heading", "Install Gude"])

    assert result.exit_code == 1
    assert "Did you mean" in result.stdout
    assert "## Install Guide" in result.stdout


# ---------------------------------------------------------------------------
# dry-run diff preview
# ---------------------------------------------------------------------------

def test_replace_dry_run_shows_diff(tmp_path: Path) -> None:
    target = tmp_path / "doc.md"
    target.write_text("# Title\nold content\n\n## Other\nstuff\n", encoding="utf-8")

    replacement = tmp_path / "new.md"
    replacement.write_text("# Title\nnew content\n", encoding="utf-8")

    result = runner.invoke(
        block_commands.app,
        ["replace", str(target), "--with-file", str(replacement), "--heading", "# Title", "--dry-run"],
    )

    assert result.exit_code == 0
    assert "DRY RUN" in result.stdout
    # File should not have been modified
    assert "old content" in target.read_text(encoding="utf-8")


def test_diff_preview_output() -> None:
    old = "line1\nline2\nline3\n"
    new = "line1\nchanged\nline3\n"

    diff_text = block_core.diff_preview(old, new, filepath="test.md")

    assert "-line2" in diff_text
    assert "+changed" in diff_text


# ---------------------------------------------------------------------------
# Fallback for unknown file types
# ---------------------------------------------------------------------------

def test_extract_unknown_extension_uses_fallback(tmp_path: Path) -> None:
    target = tmp_path / "script.xyz"
    target.write_text(dedent("""\
        def hello():
            thing1
            thing2

        def other():
            pass
    """), encoding="utf-8")

    content = block_core.extract_block(target, function="hello")

    # Fallback stops at blank line
    assert "def hello" in content
    assert "thing2" in content
    assert "def other" not in content
