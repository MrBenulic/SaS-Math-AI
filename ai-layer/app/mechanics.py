"""The fixed "Lego-block" mechanics library (ADR-008).

The translator may ONLY compose configs from these pre-defined presets — it never invents structure
or generates code. A real LLM adapter would select from the same library via constrained decoding.
"""
from __future__ import annotations

from .models import (
    Game,
    GameConfig,
    Grid,
    LineEval,
    MaxWin,
    PaytableEntry,
    Reels,
    RngConfig,
    Symbol,
    SymbolKind,
)

GRID_PRESETS: dict[str, tuple[int, int]] = {
    "3x3": (3, 3),
    "5x3": (5, 3),
    "5x4": (5, 4),
    "6x4": (6, 4),
}

# Volatility preset -> a target per-spin-return sigma (used by the solver).
VOLATILITY_PRESETS: dict[str, float] = {
    "low": 1.5,
    "medium": 3.0,
    "medium-high": 4.5,
    "high": 6.0,
    "extreme": 9.0,
}

# Base per-symbol weights and pays (count -> payout units). The solver tunes from here.
_WEIGHT_TIERS = {"WILD": 2, "SCATTER": 3, "H1": 4, "H2": 5, "L1": 8, "L2": 10, "L3": 10}
_PAY_TIERS = {
    "WILD": {3: 60, 4: 240, 5: 900},
    "H1": {3: 30, 4: 120, 5: 480},
    "H2": {3: 24, 4: 90, 5: 360},
    "L1": {3: 12, 4: 36, 5: 150},
    "L2": {3: 9, 4: 24, 5: 90},
    "L3": {3: 6, 4: 18, 5: 75},
    "SCATTER": {3: 6, 4: 30, 5: 150},  # by total scatter count
}
_REGULARS = ["H1", "H2", "L1", "L2", "L3"]


def generate_paylines(reels: int, rows: int, n: int) -> list[list[int]]:
    """Deterministic payline patterns; every row index is in [0, rows)."""
    pats: list[list[int]] = [[row] * reels for row in range(rows)]
    for offset in range(1, rows):
        pats.append([(i * offset) % rows for i in range(reels)])
        pats.append([(rows - 1 - (i * offset) % rows) for i in range(reels)])
    out: list[list[int]] = []
    i = 0
    while len(out) < n:
        out.append(pats[i % len(pats)])
        i += 1
    return out[:n]


def build_base_config(
    name: str,
    reels: int,
    rows: int,
    num_lines: int,
    has_wild: bool = True,
    has_scatter: bool = True,
    line_eval: LineEval = LineEval.left_to_right,
    cap_x: float = 5000.0,
    master_seed: int = 20260616,
) -> GameConfig:
    """Compose a valid starting game from the fixed library. The solver tunes RTP/volatility later."""
    symbols: list[Symbol] = []
    nid = 0
    wild_id = None
    if has_wild:
        wild_id = nid
        symbols.append(Symbol(id=nid, name="WILD", kind=SymbolKind.wild))
        nid += 1
    if has_scatter:
        symbols.append(Symbol(id=nid, name="SCATTER", kind=SymbolKind.scatter))
        nid += 1
    regular_ids: list[int] = []
    for nm in _REGULARS:
        regular_ids.append(nid)
        symbols.append(Symbol(id=nid, name=nm, kind=SymbolKind.regular))
        nid += 1
    if wild_id is not None:
        symbols[0].substitutes_for = list(regular_ids)

    # per-reel weights (same per reel; independent-cell weighted reels)
    weight_map = {str(s.id): _WEIGHT_TIERS[s.name] for s in symbols}
    reel_weights = [dict(weight_map) for _ in range(reels)]

    # paytable: only counts up to `reels`
    paytable: list[PaytableEntry] = []
    for s in symbols:
        tier = _PAY_TIERS.get(s.name, {})
        pays = {str(c): p for c, p in tier.items() if c <= reels}
        if pays:
            paytable.append(PaytableEntry(symbol_id=s.id, pays=pays))

    return GameConfig(
        game=Game(name=name, grid=Grid(reels=reels, rows=rows), bet_units=max(num_lines, 1)),
        symbols=symbols,
        reels=Reels(weights=reel_weights),
        paytable=paytable,
        paylines=generate_paylines(reels, rows, num_lines),
        line_eval=line_eval,
        max_win=MaxWin(cap_x=cap_x),
        rng=RngConfig(master_seed=master_seed),
    )
