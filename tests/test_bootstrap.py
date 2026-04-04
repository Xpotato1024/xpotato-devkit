from pathlib import Path

import pytest
from typer.testing import CliRunner

from devkit import bootstrap as bootstrap_module
from devkit.commands import bootstrap as bootstrap_commands
from devkit.cli import app as cli_app

runner = CliRunner()


def test_find_repo_root(tmp_path: Path) -> None:
    root = tmp_path / "repo"
    nested = root / "a" / "b"
    nested.mkdir(parents=True)
    (root / "pyproject.toml").write_text("[project]\nname = 'devkit'\n", encoding="utf-8")
    (root / "src" / "devkit").mkdir(parents=True)

    assert bootstrap_module.find_repo_root(nested) == root


def test_find_repo_root_raises_when_missing(tmp_path: Path) -> None:
    with pytest.raises(FileNotFoundError):
        bootstrap_module.find_repo_root(tmp_path)


def test_bootstrap_runs_uv_tool_commands(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    calls: list[tuple[list[str], str]] = []

    def fake_run(args, cwd, check, text, encoding, capture_output):
        calls.append((args, cwd))

        class Result:
            stdout = ""

        return Result()

    monkeypatch.setattr(bootstrap_module.subprocess, "run", fake_run)

    tool_bin = bootstrap_module.bootstrap_self(Path("D:/repo"))

    assert tool_bin == Path.home() / ".local" / "bin"
    assert calls == [
        (["uv", "tool", "install", "--force", "--editable", "."], str(Path("D:/repo"))),
        (["uv", "tool", "update-shell"], str(Path("D:/repo"))),
        (["uv", "tool", "dir", "--bin"], str(Path("D:/repo"))),
    ]


def test_get_tool_bin_prefers_uv_output(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    def fake_run(args, cwd, check, text, encoding, capture_output):
        class Result:
            stdout = "C:/Users/example/AppData/Roaming/uv/bin\n"

        return Result()

    monkeypatch.setattr(bootstrap_module.subprocess, "run", fake_run)

    assert bootstrap_module.get_tool_bin(Path("D:/repo"), tmp_path) == Path("C:/Users/example/AppData/Roaming/uv/bin")


def test_get_tool_bin_uses_provided_home_when_uv_lookup_fails(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    def fake_run(args, cwd, check, text, encoding, capture_output):
        raise FileNotFoundError("uv not found")

    monkeypatch.setattr(bootstrap_module.subprocess, "run", fake_run)

    assert bootstrap_module.get_tool_bin(Path("D:/repo"), tmp_path) == tmp_path / ".local" / "bin"


def test_bootstrap_install_self_command(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(bootstrap_commands, "find_repo_root", lambda: Path("D:/repo"))
    monkeypatch.setattr(bootstrap_commands, "bootstrap_self", lambda repo_root: Path("C:/Users/example/AppData/Roaming/uv/bin"))

    result = runner.invoke(cli_app, ["bootstrap", "install-self"])

    assert result.exit_code == 0
    assert "Bootstrap complete." in result.stdout
