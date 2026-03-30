import tomllib
from pathlib import Path
from typing import Dict, Any

def get_project_root(cwd: Path) -> Path:
    current = cwd
    for _ in range(10): # Avoid infinite recursion just in case
        if (current / ".git").is_dir() or (current / "devkit.toml").is_file():
            return current
        if current.parent == current:
            break
        current = current.parent
    return cwd

def load_config(cwd: Path = None) -> Dict[str, Any]:
    """Load configuration from devkit.toml if it exists."""
    if cwd is None:
        cwd = Path.cwd()
        
    root = get_project_root(cwd)
    config_file = root / "devkit.toml"
    
    if config_file.is_file():
        try:
            with config_file.open("rb") as f:
                return tomllib.load(f)
        except Exception:
            return {}
    return {}
