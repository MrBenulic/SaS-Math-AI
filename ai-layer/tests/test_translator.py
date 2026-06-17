"""Translator tests — rule-based default + the Groq LLM adapter (ADR-008: LLM emits only a
constrained intent; our code builds the validated config). The Groq path is tested with a mocked
HTTP response so it is deterministic and needs no network or key."""
from __future__ import annotations

import pytest

from app import translator as T
from app.models import GameConfig
from app.translator import GameIntent, GroqTranslator, build_from_intent, get_translator


def test_build_from_intent_clamps_and_normalizes():
    cfg = build_from_intent(
        GameIntent(reels=99, rows=0, num_lines=500, rtp=96, volatility="bogus", both_ways=True),
        fallback_name="x",
    )
    assert isinstance(cfg, GameConfig)
    assert cfg.game.grid.reels == 16  # clamped to MAX_REELS
    assert cfg.game.grid.rows == 1  # clamped up from 0
    assert len(cfg.paylines) == 200  # clamped to max
    assert abs((cfg.targets.rtp or 0) - 0.96) < 1e-9  # percent 96 -> fraction 0.96
    assert cfg.targets.volatility is None  # unknown preset dropped
    assert cfg.line_eval.value == "both_ways"


def test_rule_translator_still_works():
    cfg = get_translator("rule").translate("5x3, 20 lines, 96% RTP, high volatility, wild and scatter")
    assert cfg.game.grid.reels == 5 and cfg.game.grid.rows == 3
    assert len(cfg.paylines) == 20
    assert cfg.targets.rtp == 0.96 and cfg.targets.volatility == "high"


def test_groq_translator_builds_from_mocked_response(monkeypatch):
    class FakeResp:
        def raise_for_status(self):
            pass

        def json(self):
            content = (
                '{"reels":6,"rows":4,"num_lines":40,"rtp":0.95,"volatility":"high",'
                '"has_wild":true,"has_scatter":true,"both_ways":false,"name":"AI Game"}'
            )
            return {"choices": [{"message": {"content": content}}]}

    monkeypatch.setattr(T, "_load_groq_key", lambda: "test-key")
    monkeypatch.setattr(T.httpx, "post", lambda *a, **k: FakeResp())

    cfg = GroqTranslator().translate("make a 6x4 high-vol slot, 95% rtp, 40 lines")
    assert cfg.game.grid.reels == 6 and cfg.game.grid.rows == 4
    assert len(cfg.paylines) == 40
    assert cfg.targets.rtp == 0.95 and cfg.targets.volatility == "high"
    assert cfg.game.name == "AI Game"
    # the wild substitutes for the regular symbols (fixed-library invariant)
    assert any(s.kind.value == "wild" for s in cfg.symbols)


def test_groq_missing_key_raises(monkeypatch):
    monkeypatch.setattr(T, "_load_groq_key", lambda: None)
    with pytest.raises(NotImplementedError):
        GroqTranslator().translate("anything")


def test_unknown_provider_raises():
    with pytest.raises(NotImplementedError):
        get_translator("openai")
