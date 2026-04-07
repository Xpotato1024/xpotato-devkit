import json
import pytest
from pathlib import Path
from devkit.core.metrics import summarize_metrics, load_metrics

def test_summarize_metrics(tmp_path: Path):
    jsonl_file = tmp_path / ".devkit-metrics.jsonl"
    records = [
        {"ts": "2026-04-07T00:00:00Z", "cmd": "block outline", "duration_ms": 100, "brief": False, "ok": True},
        {"ts": "2026-04-07T00:00:01Z", "cmd": "block outline", "duration_ms": 200, "brief": True, "ok": True},
        {"ts": "2026-04-07T00:00:02Z", "cmd": "patch apply", "duration_ms": 500, "brief": False, "ok": False},
    ]
    with jsonl_file.open("w", encoding="utf-8") as f:
        for r in records:
            f.write(json.dumps(r) + "\n")
            
    loaded = load_metrics(jsonl_file)
    assert len(loaded) == 3
    
    summary = summarize_metrics(loaded)
    
    assert "block outline" in summary
    assert "patch apply" in summary
    
    bo = summary["block outline"]
    assert bo["count"] == 2
    assert bo["total_ms"] == 300
    assert bo["avg_ms"] == 150.0
    assert bo["brief_count"] == 1
    assert bo["success_count"] == 2
    
    pa = summary["patch apply"]
    assert pa["count"] == 1
    assert pa["success_count"] == 0
