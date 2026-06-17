"""Math-sheet (.xlsx) exporter tests — the human-readable deliverable opens as a valid workbook
with the expected sheets and key facts."""
from __future__ import annotations

import io

import openpyxl

from app.mathsheet import build_math_sheet_xlsx

CONFIG = {
    "schema_version": "0.1.0",
    "game": {"name": "Demo Slot", "grid": {"reels": 3, "rows": 1}, "bet_units": 1, "denomination_scale": 1},
    "symbols": [
        {"id": 0, "name": "WILD", "kind": "wild", "substitutes_for": [2]},
        {"id": 1, "name": "S", "kind": "scatter"},
        {"id": 2, "name": "A", "kind": "regular"},
    ],
    "reels": {"weights": [{"0": 1, "1": 1, "2": 3}, {"0": 1, "1": 1, "2": 3}, {"0": 1, "1": 1, "2": 3}]},
    "paytable": [{"symbol_id": 2, "pays": {"2": 5, "3": 20}}, {"symbol_id": 1, "pays": {"2": 3}}],
    "paylines": [[0, 0, 0]],
    "line_eval": "left_to_right",
    "max_win": {"cap_x": 1000},
    "rng": {"generator": "chacha", "master_seed": 1},
}

RESULTS = {
    "exact_rtp": 0.95,
    "line_rtp": 0.87,
    "scatter_rtp": 0.08,
    "exact_line_ratio_num": "3",
    "exact_line_ratio_den": "4",
    "exact_combos": "27",
    "capped_rtp": 0.95,
    "mc_rtp": 0.949,
    "mc_ci95": 0.001,
    "hit_frequency": 0.42,
    "volatility": 3.9,
    "max_win_x": 200.0,
    "spins": 300000,
    "win_distribution": [
        {"label": "0x (no win)", "probability": 0.48, "count": "144000"},
        {"label": "1x-2x", "probability": 0.15, "count": "45000"},
    ],
}


def test_builds_valid_workbook_with_expected_sheets():
    data = build_math_sheet_xlsx(CONFIG, RESULTS, "sha256:abc123")
    assert isinstance(data, bytes) and len(data) > 1000
    wb = openpyxl.load_workbook(io.BytesIO(data))  # raises if not a valid xlsx
    assert wb.sheetnames == ["Summary", "Symbols", "Reels", "Paytable", "Win distribution"]


def test_summary_carries_key_facts():
    wb = openpyxl.load_workbook(io.BytesIO(build_math_sheet_xlsx(CONFIG, RESULTS, "sha256:abc123")))
    text = "\n".join(
        str(c.value) for row in wb["Summary"].iter_rows() for c in row if c.value is not None
    )
    assert "SaS·Math — Game Math Sheet" in text
    assert "Demo Slot" in text
    assert "95.0000%" in text  # exact RTP rendered
    assert "sha256:abc123" in text  # fingerprint present
    assert "3 / 4" in text  # integer ratio
    assert "ChaCha8" in text  # RNG reported as the engine's ACTUAL primitive, not the config string


def test_paytable_and_distribution_populated():
    wb = openpyxl.load_workbook(io.BytesIO(build_math_sheet_xlsx(CONFIG, RESULTS, "sha256:abc123")))
    pay = wb["Paytable"]
    assert pay.cell(row=1, column=1).value == "Symbol"
    assert pay.max_row >= 3  # header + 2 paytable rows
    dist = wb["Win distribution"]
    assert dist.cell(row=2, column=1).value == "0x (no win)"
