import tomllib
from pathlib import Path
from typing import Any, Dict, Optional

def get_project_root(cwd: Path) -> Path:
    current = cwd
    while True:
        if (current / ".git").is_dir() or (current / "devkit.toml").is_file():
            return current
        if current.parent == current:
            break
        current = current.parent
    return cwd

def load_config(cwd: Optional[Path] = None) -> Dict[str, Any]:
    """Load configuration from devkit.toml if it exists."""
    if cwd is None:
        cwd = Path.cwd()

    root = get_project_root(cwd)
    config_file = root / "devkit.toml"

    if config_file.is_file():
        with config_file.open("rb") as f:
            try:
                return tomllib.load(f)
            except tomllib.TOMLDecodeError as exc:
                raise ValueError(f"Invalid TOML in {config_file}: {exc}") from exc
    return {}
