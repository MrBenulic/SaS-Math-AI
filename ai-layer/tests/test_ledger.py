"""Published-math ledger tests — content-addressed, upsert-by-fingerprint, durable index."""
from __future__ import annotations

from app import ledger


def _artifact(chash: str, name: str, rtp: float) -> dict:
    return {
        "config_hash": chash,
        "schema_version": "0.1.0",
        "config": {"game": {"name": name}},
        "results": {"exact_rtp": rtp, "exact_line_ratio_num": "3", "exact_line_ratio_den": "4"},
    }


def test_record_then_read(tmp_path):
    entry = ledger.record(_artifact("sha256:aaa", "Game A", 0.95), tmp_path)
    assert entry["config_hash"] == "sha256:aaa"
    assert entry["exact_line_ratio"] == "3/4"

    index = ledger.read_index(tmp_path)
    assert len(index) == 1
    assert index[0]["name"] == "Game A"
    # the full artifact file was written
    assert (tmp_path / "sha256_aaa.json").exists()


def test_upsert_same_hash(tmp_path):
    ledger.record(_artifact("sha256:aaa", "Game A", 0.95), tmp_path)
    ledger.record(_artifact("sha256:aaa", "Game A (retuned)", 0.96), tmp_path)
    index = ledger.read_index(tmp_path)
    assert len(index) == 1  # same hash -> updated in place, not duplicated
    assert index[0]["name"] == "Game A (retuned)"
    assert index[0]["exact_rtp"] == 0.96


def test_distinct_hashes_accumulate(tmp_path):
    ledger.record(_artifact("sha256:aaa", "A", 0.95), tmp_path)
    ledger.record(_artifact("sha256:bbb", "B", 0.94), tmp_path)
    hashes = {e["config_hash"] for e in ledger.read_index(tmp_path)}
    assert hashes == {"sha256:aaa", "sha256:bbb"}


def test_read_index_empty_when_absent(tmp_path):
    assert ledger.read_index(tmp_path / "nope") == []
