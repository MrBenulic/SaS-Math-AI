"""Bridge to the Rust math engine via the `engine` JSON binary (stdin/stdout).

This is the M3 integration; PyO3 in-process bindings (ADR-007) are the planned production upgrade.
Override the binary path with the SAS_ENGINE_BIN environment variable.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any, Optional

_DEFAULT_BIN = Path(__file__).resolve().parents[2] / "core-math" / "target" / "release" / "engine"


def engine_bin() -> str:
    return os.environ.get("SAS_ENGINE_BIN", str(_DEFAULT_BIN))


def run_engine(request: dict[str, Any], timeout: float = 180.0) -> dict[str, Any]:
    binary = engine_bin()
    if not Path(binary).exists():
        raise RuntimeError(
            f"engine binary not found at {binary}; build it: "
            "(cd core-math && cargo build --release --bin engine)"
        )
    proc = subprocess.run(
        [binary],
        input=json.dumps(request),
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stdout.strip() or proc.stderr.strip() or "engine failed")
    out = json.loads(proc.stdout)
    if isinstance(out, dict) and "error" in out:
        raise RuntimeError(out["error"])
    return out


def simulate(config: dict[str, Any], spins: int = 2_000_000) -> dict[str, Any]:
    return run_engine({"command": "simulate", "config": config, "spins": spins})


def tune(
    config: dict[str, Any],
    target_rtp: float,
    generations: int = 200,
    target_volatility: Optional[float] = None,
    target_hit_frequency: Optional[float] = None,
) -> dict[str, Any]:
    req: dict[str, Any] = {
        "command": "tune",
        "config": config,
        "target_rtp": target_rtp,
        "generations": generations,
    }
    if target_volatility is not None:
        req["target_volatility"] = target_volatility
    if target_hit_frequency is not None:
        req["target_hit_frequency"] = target_hit_frequency
    return run_engine(req)
