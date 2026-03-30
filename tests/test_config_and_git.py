from pathlib import Path

import pytest
from typer.testing import CliRunner

from devkit.commands import encoding as encoding_commands
from devkit.commands import git as git_commands
from devkit.core import config as config_module
from devkit.core import git as git_core

runner = CliRunner()


def test_get_project_root_walks_past_depth_limit(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    nested = root
    for index in range(12):
        nested = nested / f"level{index}"
    nested.mkdir(parents=True)
    (root / "devkit.toml").write_text("[git]\nlang = \"en\"\n", encoding="utf-8")

    assert config_module.get_project_root(nested) == root


def test_load_config_raises_on_invalid_toml(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    root.mkdir()
    (root / "devkit.toml").write_text("invalid = [\n", encoding="utf-8")

    with pytest.raises(ValueError, match="Invalid TOML"):
        config_module.load_config(root)


def test_generate_commit_template_is_compact_and_structured(monkeypatch: pytest.MonkeyPatch) -> None:
    summary = {
        "files": [
            {"path": "src/app.py", "additions": 12, "deletions": 3, "is_binary": False},
            {"path": "assets/logo.png", "additions": 0, "deletions": 0, "is_binary": True},
        ],
        "total_additions": 12,
        "total_deletions": 3,
    }
    monkeypatch.setattr(git_core, "summarize_diff", lambda staged=True: summary)

    def unexpected_git_call(*args, **kwargs):  # pragma: no cover - guard rail
        raise AssertionError("commit template should not fetch the full diff")

    monkeypatch.setattr(git_core, "run_git_command", unexpected_git_call)

    content = git_core.generate_commit_template(staged=True, lang="en")

    assert "You are drafting a Git commit message." in content
    assert "# Diff summary: 12 additions, 3 deletions across 2 file(s)." in content
    assert "# - src/app.py (+12/-3)" in content
    assert "# - assets/logo.png (binary)" in content
    assert "Full diff" not in content


def test_safe_push_uses_explicit_remote(monkeypatch: pytest.MonkeyPatch) -> None:
    calls = {}

    monkeypatch.setattr(git_commands, "check_safe_branch", lambda: None)
    monkeypatch.setattr(git_commands, "check_upstream", lambda: False)
    monkeypatch.setattr(git_commands, "get_current_branch", lambda: "feature/demo")

    def fake_run(args, **kwargs):
        calls["args"] = args

        class Result:
            returncode = 0

        return Result()

    monkeypatch.setattr(git_commands.subprocess, "run", fake_run)

    result = runner.invoke(git_commands.app, ["safe-push", "--yes", "--remote", "upstream"])

    assert result.exit_code == 0
    assert calls["args"] == ["git", "push", "-u", "upstream", "feature/demo"]


def test_generate_pr_template_is_compact_and_structured(monkeypatch: pytest.MonkeyPatch) -> None:
    calls = []

    def fake_run_git_command(args):
        calls.append(args)
        if args[:2] == ["diff", "--stat"]:
            return "src/app.py | 3 +--"
        if args[:2] == ["log", "--oneline"]:
            return "abc123 Fix thing"
        raise AssertionError(f"unexpected git call: {args}")

    monkeypatch.setattr(git_core, "run_git_command", fake_run_git_command)

    content = git_core.generate_pr_template(base="main", lang="en")

    assert "You are drafting a pull request body." in content
    assert "* Summarize the change in 1-2 sentences." in content
    assert "# - abc123 Fix thing" in content
    assert "# - src/app.py | 3 +--" in content
    assert "Fill in the sections above." in content
    assert calls == [["diff", "--stat", "main...HEAD"], ["log", "--oneline", "main..HEAD"]]


def test_encoding_check_reports_invalid_config(monkeypatch: pytest.MonkeyPatch) -> None:
    def broken_config():
        raise ValueError("Invalid TOML in devkit.toml")

    monkeypatch.setattr(encoding_commands, "load_config", broken_config)

    result = runner.invoke(encoding_commands.app, ["check", "README.md"])

    assert result.exit_code == 1
    assert "Invalid TOML" in result.stdout
