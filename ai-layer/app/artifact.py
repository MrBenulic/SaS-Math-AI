"""Published-math artifact + config fingerprint — the certification-fingerprint foothold (rank 5).

A validated config is itself the certification artifact (ADR-004): it reproduces identical math
forever and its money fields are integers (ADR-002). We canonicalize it (deterministic JSON: sorted
keys, compact, UTF-8) and hash it with SHA-256 (a regulator-recognized digest) to get a stable
`config_hash`. A *published artifact* bundles that fingerprint with the engine results and version
stamps into a reproducible audit record. The same hash lets a presentation binding prove exactly
which config its art targets (see `binding.py`).

This is intentionally a thin, zero-math-risk layer over already-locked decisions — the certification
*claim* still rests on the Rust engine's exact math, not on this wrapper.
"""
from __future__ import annotations

import hashlib
import json
from typing import Any

from .models import GameConfig

ARTIFACT_VERSION = "0.1.0"
ENGINE_VERSION = "core-math/0.1.0"


def _canonical_dict(config: GameConfig) -> dict[str, Any]:
    """The config as a plain JSON-able dict, with unset optional fields dropped (stable shape)."""
    return config.model_dump(mode="json", exclude_none=True)


def canonical_config_bytes(config: GameConfig) -> bytes:
    """Deterministic byte serialization of a validated config (sorted keys, compact, UTF-8)."""
    return json.dumps(
        _canonical_dict(config), sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def config_hash(config: GameConfig) -> str:
    """SHA-256 fingerprint of the canonical config, prefixed with the algorithm (e.g. `sha256:ab…`)."""
    return "sha256:" + hashlib.sha256(canonical_config_bytes(config)).hexdigest()


def build_published_artifact(
    config: GameConfig,
    results: dict[str, Any],
    engine_version: str = ENGINE_VERSION,
) -> dict[str, Any]:
    """Assemble a reproducible audit record: fingerprint + canonical config + engine results + stamps.

    `results` is the engine's metrics dict (exact RTP is the source of truth; MC is reported with CI).
    The stored `config` equals what was hashed, so the record is self-verifying.
    """
    return {
        "artifact_version": ARTIFACT_VERSION,
        "schema_version": config.schema_version,
        "engine_version": engine_version,
        "config_hash": config_hash(config),
        "config": _canonical_dict(config),
        "results": results,
    }
