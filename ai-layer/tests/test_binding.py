"""Art-binding (presentation) validator tests — the bidirectional completeness check that makes
'art + certified math = one package' a hard guarantee."""
from __future__ import annotations

from app.binding import (
    EVENT_TYPES,
    EventBinding,
    PresentationBind,
    SymbolBinding,
    validate_binding,
)
from app.models import GameConfig


def make_config() -> GameConfig:
    return GameConfig(
        **{
            "schema_version": "0.1.0",
            "game": {"name": "t", "grid": {"reels": 3, "rows": 1}, "bet_units": 1},
            "symbols": [
                {"id": 0, "name": "A", "kind": "regular"},
                {"id": 1, "name": "W", "kind": "wild"},
            ],
            "reels": {"strips": [[0, 1], [0, 1], [0, 1]]},
            "paytable": [{"symbol_id": 0, "pays": {"3": 9}}],
            "paylines": [[0, 0, 0]],
            "line_eval": "left_to_right",
            "max_win": {"cap_x": 1000},
            "rng": {"generator": "chacha", "master_seed": 1},
        }
    )


def full_binding() -> PresentationBind:
    return PresentationBind(
        symbols=[
            SymbolBinding(symbol_id=0, skeleton="a.json", land_anim="a_land"),
            SymbolBinding(symbol_id=1, spritesheet={"sheet": "w.png", "frames": ["w0", "w1"]}),
        ],
        events=[EventBinding(event_type=et, anim=f"{et}_anim") for et in EVENT_TYPES],
    )


def test_complete_binding_is_valid():
    report = validate_binding(make_config(), full_binding())
    assert report.valid, report.errors
    assert report.errors == []


def test_missing_symbol_is_flagged():
    b = full_binding()
    b.symbols = [s for s in b.symbols if s.symbol_id != 1]  # drop the wild
    report = validate_binding(make_config(), b)
    assert not report.valid
    assert report.missing_symbols == [1]


def test_missing_event_is_flagged():
    b = full_binding()
    b.events = [e for e in b.events if e.event_type != "scatter_win"]
    report = validate_binding(make_config(), b)
    assert not report.valid
    assert "scatter_win" in report.missing_events


def test_unknown_symbol_id_is_flagged():
    b = full_binding()
    b.symbols.append(SymbolBinding(symbol_id=99, skeleton="ghost.json"))
    report = validate_binding(make_config(), b)
    assert not report.valid
    assert report.unknown_symbols == [99]


def test_unknown_event_type_is_flagged():
    b = full_binding()
    b.events.append(EventBinding(event_type="teleport", anim="x"))
    report = validate_binding(make_config(), b)
    assert not report.valid
    assert report.unknown_events == ["teleport"]


def test_symbol_without_renderable_is_flagged():
    b = full_binding()
    # bind symbol 0 with only a sound — no skeleton/region/spritesheet
    b.symbols = [SymbolBinding(symbol_id=0, sound_cue="ping"), b.symbols[1]]
    report = validate_binding(make_config(), b)
    assert not report.valid
    assert report.unrenderable_symbols == [0]
