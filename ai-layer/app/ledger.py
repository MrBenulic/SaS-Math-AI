"""Published-math ledger — a file-backed index of published artifacts.

The first persistence step for M3 (Postgres/SQLite is the planned upgrade). Each published artifact is
written under its `config_hash`, and a compact entry is upserted into `index.json` — a durable,
content-addressed record of every certified game. Writes are serialized (in-process lock) and atomic
(temp file + os.replace), and a corrupt index is preserved rather than silently overwritten.
"""
from __future__ import annotations

import json
import os
import threading
from pathlib import Path
from typing import Any, Union

DEFAULT_DIR = Path(__file__).resolve().parents[1] / "published"

# Serialize the read-modify-write of index.json within the process (FastAPI runs sync handlers in a
# threadpool, so concurrent /publish calls could otherwise interleave and lose entries).
_LOCK = threading.Lock()


def _safe_name(config_hash: str) -> str:
    # "sha256:abcd…" -> "sha256_abcd….json"; strip path separators defensively.
    return config_hash.replace(":", "_").replace("/", "_").replace("\\", "_").replace("..", "_") + ".json"


def _atomic_write(path: Path, text: str) -> None:
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(text)
    os.replace(tmp, path)  # atomic rename on POSIX


def read_index(directory: Union[Path, str] = DEFAULT_DIR) -> list[dict[str, Any]]:
    """Lenient read for callers/UI: returns [] if absent or unparseable (does not mutate)."""
    idx = Path(directory) / "index.json"
    if not idx.exists():
        return []
    try:
        data = json.loads(idx.read_text() or "[]")
        return data if isinstance(data, list) else []
    except json.JSONDecodeError:
        return []


def _read_index_for_update(d: Path) -> list[dict[str, Any]]:
    """Read for record(): if the index exists but is corrupt, PRESERVE it (back it up) instead of
    silently overwriting — never lose prior published records."""
    idx = d / "index.json"
    if not idx.exists():
        return []
    raw = idx.read_text()
    if not raw.strip():
        return []
    try:
        data = json.loads(raw)
        return data if isinstance(data, list) else []
    except json.JSONDecodeError:
        os.replace(idx, d / "index.json.corrupt")  # keep the bad file for inspection
        return []


def _line_ratio(results: dict[str, Any]) -> Union[str, None]:
    num = results.get("exact_line_ratio_num")
    den = results.get("exact_line_ratio_den")
    # ways games emit "0"/"0" (RTP is a closed-form expectation, not an integer ratio) — record None.
    if not num or num == "0" or not den or den == "0":
        return None
    return f"{num}/{den}"


def record(artifact: dict[str, Any], directory: Union[Path, str] = DEFAULT_DIR) -> dict[str, Any]:
    """Write the full artifact and upsert a compact entry into index.json (locked + atomic)."""
    d = Path(directory)
    chash = artifact.get("config_hash", "unknown")
    results = artifact.get("results", {}) or {}
    entry = {
        "config_hash": chash,
        "name": (artifact.get("config", {}) or {}).get("game", {}).get("name"),
        "schema_version": artifact.get("schema_version"),
        "exact_rtp": results.get("exact_rtp"),
        "exact_line_ratio": _line_ratio(results),
        "artifact_file": _safe_name(chash),
    }
    with _LOCK:
        d.mkdir(parents=True, exist_ok=True)
        _atomic_write(d / _safe_name(chash), json.dumps(artifact, indent=2))
        index = [e for e in _read_index_for_update(d) if e.get("config_hash") != chash]  # upsert
        index.append(entry)
        _atomic_write(d / "index.json", json.dumps(index, indent=2))
    return entry
