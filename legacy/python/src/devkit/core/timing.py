"""Lightweight timing infrastructure for devkit commands.

Usage::

    from devkit.core.timing import get_context, timed

    with timed("git"):
        subprocess.run(...)

The CLI callback creates / finalises the context; core modules just
call ``timed()`` and no-op if timing is disabled.
"""

from __future__ import annotations

import contextlib
import json
import sys
import time
import threading
from dataclasses import dataclass, field
from typing import Optional


# ---------------------------------------------------------------------------
# Timer
# ---------------------------------------------------------------------------

@dataclass
class Timer:
    """Simple start/stop timer."""
    _start: float = 0.0
    _elapsed: float = 0.0
    _running: bool = False

    def start(self) -> None:
        self._start = time.perf_counter()
        self._running = True

    def stop(self) -> None:
        if self._running:
            self._elapsed += time.perf_counter() - self._start
            self._running = False

    @property
    def elapsed_ms(self) -> float:
        extra = (time.perf_counter() - self._start) if self._running else 0.0
        return (self._elapsed + extra) * 1000


# ---------------------------------------------------------------------------
# TimingContext — collects categorical timers
# ---------------------------------------------------------------------------

CATEGORIES = ("git", "io", "parse", "render")

@dataclass
class TimingContext:
    """Accumulates timing data for a single command invocation."""
    total: Timer = field(default_factory=Timer)
    categories: dict[str, Timer] = field(default_factory=dict)

    def __post_init__(self) -> None:
        for cat in CATEGORIES:
            self.categories.setdefault(cat, Timer())

    def start(self) -> None:
        self.total.start()

    def stop(self) -> None:
        self.total.stop()

    def get_timer(self, category: str) -> Timer:
        if category not in self.categories:
            self.categories[category] = Timer()
        return self.categories[category]

    # -- Reporting ----------------------------------------------------------

    def to_dict(self) -> dict[str, float]:
        d: dict[str, float] = {"total_ms": round(self.total.elapsed_ms, 1)}
        for cat in CATEGORIES:
            ms = round(self.categories[cat].elapsed_ms, 1)
            if ms > 0:
                d[f"{cat}_ms"] = ms
        return d

    def format_human(self) -> str:
        d = self.to_dict()
        total = d["total_ms"]
        parts = [f"{total:.0f}ms"]
        detail = []
        for cat in CATEGORIES:
            key = f"{cat}_ms"
            if key in d and d[key] > 0:
                detail.append(f"{cat}: {d[key]:.0f}ms")
        if detail:
            parts.append(f"({', '.join(detail)})")
        return f"[time] {' '.join(parts)}"

    def format_json(self) -> str:
        return json.dumps(self.to_dict())


# ---------------------------------------------------------------------------
# Thread-local context storage
# ---------------------------------------------------------------------------

_local = threading.local()


def set_context(ctx: Optional[TimingContext]) -> None:
    _local.timing_ctx = ctx


def get_context() -> Optional[TimingContext]:
    return getattr(_local, "timing_ctx", None)


# ---------------------------------------------------------------------------
# Public helpers
# ---------------------------------------------------------------------------

@contextlib.contextmanager
def timed(category: str):
    """Context manager that records elapsed time under *category*.

    No-ops gracefully if no TimingContext is active.
    """
    ctx = get_context()
    if ctx is None:
        yield
        return
    timer = ctx.get_timer(category)
    timer.start()
    try:
        yield
    finally:
        timer.stop()


def emit_timing(mode: str) -> None:
    """Write timing result to stderr.  *mode* is ``'human'`` or ``'json'``."""
    ctx = get_context()
    if ctx is None:
        return
    ctx.stop()
    if mode == "json":
        print(ctx.format_json(), file=sys.stderr)
    else:
        print(ctx.format_human(), file=sys.stderr)
