#!/usr/bin/env python3
"""Generate examples/sample_5x3.json — a reference 5x3 lines game for the M1 engine.

The strip composition is spread evenly (largest-remainder placement) so scatters/wilds are not
clustered. Re-run after editing WEIGHTS/paytable. Exact==MC holds for any arrangement.
"""
import json
import os

SYMS = [
    {"id": 0, "name": "WILD", "kind": "wild", "substitutes_for": [2, 3, 4, 5, 6]},
    {"id": 1, "name": "SCATTER", "kind": "scatter"},
    {"id": 2, "name": "H1", "kind": "regular"},
    {"id": 3, "name": "H2", "kind": "regular"},
    {"id": 4, "name": "L1", "kind": "regular"},
    {"id": 5, "name": "L2", "kind": "regular"},
    {"id": 6, "name": "L3", "kind": "regular"},
]
# per-reel composition (strip length = sum = 32)
WEIGHTS = {0: 2, 1: 3, 2: 4, 3: 5, 4: 6, 5: 6, 6: 6}


def spread_strip(weights):
    placed = {s: 0 for s in weights}
    strip = []
    for _ in range(sum(weights.values())):
        best, bestval = None, 9.9
        for s in sorted(weights):
            if placed[s] < weights[s]:
                val = (placed[s] + 1) / weights[s]
                if val < bestval - 1e-12:
                    bestval, best = val, s
        strip.append(best)
        placed[best] += 1
    return strip


strip = spread_strip(WEIGHTS)
reels = [strip[:] for _ in range(5)]

paylines = [
    [1, 1, 1, 1, 1], [0, 0, 0, 0, 0], [2, 2, 2, 2, 2], [0, 1, 2, 1, 0], [2, 1, 0, 1, 2],
    [1, 0, 1, 0, 1], [1, 2, 1, 2, 1], [0, 0, 1, 2, 2], [2, 2, 1, 0, 0], [1, 1, 0, 1, 2],
]

# Base pays, scaled to land near ~95% RTP. (Precise RTP targeting is the M2 solver's job;
# this is just a believable hand-built reference game.)
SCALE = 3
_base = [
    (0, {"3": 20, "4": 80, "5": 300}),   # WILD
    (1, {"3": 2, "4": 10, "5": 50}),     # SCATTER (by total count)
    (2, {"3": 10, "4": 40, "5": 160}),   # H1
    (3, {"3": 8, "4": 25, "5": 120}),    # H2
    (4, {"3": 4, "4": 12, "5": 50}),     # L1
    (5, {"3": 3, "4": 8, "5": 30}),      # L2
    (6, {"3": 2, "4": 6, "5": 25}),      # L3
]
paytable = [
    {"symbol_id": sid, "pays": {k: v * SCALE for k, v in pays.items()}}
    for sid, pays in _base
]

cfg = {
    "schema_version": "0.1.0",
    "game": {
        "name": "Sample 5x3 Lines (M1 reference)",
        "grid": {"reels": 5, "rows": 3},
        "bet_units": 20,
        "denomination_scale": 100,
    },
    "symbols": SYMS,
    "reels": {"strips": reels},
    "paytable": paytable,
    "paylines": paylines,
    "line_eval": "left_to_right",
    "max_win": {"cap_x": 100000},
    "rng": {"generator": "chacha", "master_seed": 20260616},
}

out = os.path.join(os.path.dirname(__file__), "sample_5x3.json")
with open(out, "w") as f:
    json.dump(cfg, f, indent=2)
print("wrote", out, "| strip len", len(strip),
      "| composition", {SYMS[s]["name"]: WEIGHTS[s] for s in WEIGHTS})
