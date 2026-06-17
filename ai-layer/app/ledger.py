"""Published-math ledger — a file-backed index of published artifacts.

The first persistence step for M3 (Postgres/SQLite is the planned upgrade). Each published artifact is
written under its `config_hash`, and a compact entry is upserted into `index.json` — so a studio has a
durable, content-addressed record of every certified game it has published. Keyed by the SHA-256
fingerprint (see `artifact.py`), so re-publishing the same config updates in place rather than
duplicating.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Union

DEFAULT_DIR = Path(__file__).resolve().parents[1] / "published"


def _safe_name(config_hash: str) -> str:
    # "sha256:abcd…" -> "sha256_abcd….json" (filesystem-safe)
    return config_hash.replace(":", "_").replace("/", "_") + ".json"


def read_index(directory: Union[Path, str] = DEFAULT_DIR) -> list[dict[str, Any]]:
    idx = Path(directory) / "index.json"
    if not idx.exists():
        return []
    try:
        data = json.loads(idx.read_text())
        return data if isinstance(data, list) else []
    except json.JSONDecodeError:
        return []


def record(artifact: dict[str, Any], directory: Union[Path, str] = DEFAULT_DIR) -> dict[str, Any]:
    """Write the full artifact and upsert a compact entry into index.json. Returns the entry."""
    d = Path(directory)
    d.mkdir(parents=True, exist_ok=True)
    chash = artifact.get("config_hash", "unknown")
    (d / _safe_name(chash)).write_text(json.dumps(artifact, indent=2))

    results = artifact.get("results", {}) or {}
    entry = {
        "config_hash": chash,
        "name": (artifact.get("config", {}) or {}).get("game", {}).get("name"),
        "schema_version": artifact.get("schema_version"),
        "exact_rtp": results.get("exact_rtp"),
        "exact_line_ratio": (
            f"{results.get('exact_line_ratio_num')}/{results.get('exact_line_ratio_den')}"
            if results.get("exact_line_ratio_num")
            else None
        ),
        "artifact_file": _safe_name(chash),
    }
    index = [e for e in read_index(d) if e.get("config_hash") != chash]  # upsert
    index.append(entry)
    (d / "index.json").write_text(json.dumps(index, indent=2))
    return entry
