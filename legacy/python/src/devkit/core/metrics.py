import json
import logging
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

from devkit.core.config import get_project_root


def get_metrics_file(cwd: Optional[Path] = None) -> Optional[Path]:
    """Return the metrics file path if metrics collection is enabled, else None."""
    from devkit.core.config import load_config

    try:
        config = load_config(cwd)
        metrics_cfg = config.get("metrics", {})
        if not metrics_cfg.get("enabled", False):
            return None
        
        path_str = metrics_cfg.get("path", ".devkit-metrics.jsonl")
        root = get_project_root(cwd or Path.cwd())
        return root / path_str
    except Exception:
        return None


def record_metric(
    cmd: str,
    duration_ms: float,
    brief: bool,
    ok: bool,
    cwd: Optional[Path] = None
) -> None:
    """Record a command execution metric to the JSONL file."""
    try:
        metrics_file = get_metrics_file(cwd)
        if not metrics_file:
            return

        record = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "cmd": cmd,
            "duration_ms": duration_ms,
            "brief": brief,
            "ok": ok
        }

        # Append to jsonl
        with metrics_file.open("a", encoding="utf-8") as f:
            f.write(json.dumps(record) + "\n")
    except Exception as e:
        # Silently fail for metrics collection errors
        pass


def load_metrics(filepath: Path) -> List[Dict[str, Any]]:
    """Load all metrics from the JSONL file."""
    records = []
    if not filepath.is_file():
        return records

    with filepath.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return records


def summarize_metrics(records: List[Dict[str, Any]]) -> Dict[str, Dict[str, Any]]:
    """Summarize metrics by command."""
    summary: Dict[str, Dict[str, Any]] = {}
    for r in records:
        cmd = r.get("cmd", "unknown")
        if cmd not in summary:
            summary[cmd] = {
                "count": 0,
                "total_ms": 0.0,
                "success_count": 0,
                "brief_count": 0,
            }
        
        st = summary[cmd]
        st["count"] += 1
        st["total_ms"] += r.get("duration_ms", 0.0)
        if r.get("ok", False):
            st["success_count"] += 1
        if r.get("brief", False):
            st["brief_count"] += 1

    # compute averages
    for cmd, st in summary.items():
        st["avg_ms"] = st["total_ms"] / st["count"] if st["count"] > 0 else 0.0
        st["success_rate"] = st["success_count"] / st["count"] if st["count"] > 0 else 0.0

    return summary
