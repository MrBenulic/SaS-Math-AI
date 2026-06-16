"""Published-math artifact + config-fingerprint tests (the certification-fingerprint foothold)."""
from __future__ import annotations

from app.artifact import (
    build_published_artifact,
    canonical_config_bytes,
    config_hash,
)
from app.models import GameConfig


def make_config(pay_3: int = 9) -> GameConfig:
    return GameConfig(
        **{
            "schema_version": "0.1.0",
            "game": {"name": "t", "grid": {"reels": 3, "rows": 1}, "bet_units": 1},
            "symbols": [{"id": 0, "name": "A", "kind": "regular"}],
            "reels": {"strips": [[0], [0], [0]]},
            "paytable": [{"symbol_id": 0, "pays": {"3": pay_3}}],
            "paylines": [[0, 0, 0]],
            "line_eval": "left_to_right",
            "max_win": {"cap_x": 1000},
            "rng": {"generator": "chacha", "master_seed": 1},
        }
    )


def test_fingerprint_is_deterministic():
    a = config_hash(make_config())
    b = config_hash(make_config())
    assert a == b
    assert a.startswith("sha256:")


def test_canonical_bytes_are_stable_and_sorted():
    # Two independently-built equal configs canonicalize to identical bytes.
    assert canonical_config_bytes(make_config()) == canonical_config_bytes(make_config())


def test_changing_the_math_changes_the_fingerprint():
    # A different payout is a different certified game -> a different fingerprint.
    assert config_hash(make_config(pay_3=9)) != config_hash(make_config(pay_3=10))


def test_published_artifact_is_self_describing():
    cfg = make_config()
    art = build_published_artifact(cfg, results={"exact_rtp": 0.5})
    assert art["config_hash"] == config_hash(cfg)
    assert art["schema_version"] == "0.1.0"
    assert "engine_version" in art and "artifact_version" in art
    assert art["results"] == {"exact_rtp": 0.5}
    # the stored config is exactly what was hashed
    assert build_published_artifact(GameConfig(**art["config"]), results={})["config_hash"] == art["config_hash"]
