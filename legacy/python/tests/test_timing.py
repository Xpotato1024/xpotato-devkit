import json
import logging
from devkit.core.timing import TimingContext, set_context, timed

def test_timing_context_basic():
    ctx = TimingContext()
    ctx.start()
    set_context(ctx)

    import time
    with timed("io"):
        time.sleep(0.002)

    with timed("git"):
        time.sleep(0.002)

    ctx.stop()
    d = ctx.to_dict()
    assert "total_ms" in d
    assert "io_ms" in d
    assert "git_ms" in d
    assert "parse_ms" not in d

    human = ctx.format_human()
    assert "[time]" in human
    assert "ms" in human

    jstr = ctx.format_json()
    assert "total_ms" in json.loads(jstr)
    
    set_context(None)

def test_timed_without_context_noops():
    set_context(None)
    with timed("io"):
        pass  # should not raise
