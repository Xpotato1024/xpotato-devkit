from typer.testing import CliRunner

from devkit.cli import app

runner = CliRunner()

def test_help():
    result = runner.invoke(app, ["--help"])
    assert result.exit_code == 0
    assert "Repo-agnostic CLI" in result.stdout

def test_encoding_check_missing_file():
    result = runner.invoke(app, ["encoding", "check", "non-existent-file-xyz.txt"])
    assert result.exit_code == 1

def test_diff_summarize_help():
    result = runner.invoke(app, ["diff", "summarize", "--help"])
    assert result.exit_code == 0
    assert "--base" in result.stdout
    assert "--head" in result.stdout

def test_block_extract_missing_file():
    result = runner.invoke(app, ["block", "extract", "non-existent-file-xyz.txt", "--lines", "1-2"])
    assert result.exit_code == 1

def test_patch_apply_missing_file():
    result = runner.invoke(app, ["patch", "apply", "--patch-file", "non-existent-xyz.patch"])
    assert result.exit_code == 1
def test_git_commit_message_help():
    result = runner.invoke(app, ["git", "commit-message", "--help"])
    assert result.exit_code == 0
    assert "--commits" in result.stdout
    assert "--base" in result.stdout

def test_git_pr_body_help():
    result = runner.invoke(app, ["git", "pr-body", "--help"])
    assert result.exit_code == 0
    assert "--staged" in result.stdout
    assert "--commits" in result.stdout

def test_git_safe_push_help():
    result = runner.invoke(app, ["git", "safe-push", "--help"])
    assert result.exit_code == 0
