"""M3 ai-layer tests:
 1. translator fuzz — every prompt produces a schema-valid config (ADR-008)
 2. endpoints — translate / validate
 3. integration — simulate + the full prompt->JSON->simulate->tune pipeline (shells to the Rust engine)
"""
from app.main import app
from app.models import GameConfig
from app.translator import RuleBasedTranslator
from fastapi.testclient import TestClient

client = TestClient(app)

PROMPTS = [
    "5x3 slot, 20 paylines, 96% RTP, high volatility, with wild and scatter",
    "make a 3x3 fruit machine, low volatility, 5 lines, 95% rtp",
    "6x4 adventure slot, medium volatility, scatter only, 30 lines",
    "simple 5x3, 25 lines",
    "5x4 slot, both ways, medium-high volatility, 40 lines, 94% rtp, wild scatter",
    "classic slot no scatter, 5x3, 10 lines",
    "extreme volatility 5x3 with wilds, 20 lines, rtp 96%",
    "a relaxing 3x3 game, low vol, 8 lines",
    "5x3, 15 lines, return 90%",
    "big 6x4 slot, 50 lines, high volatility, wild and scatter, 96.5% rtp",
]


def test_translator_always_valid():
    t = RuleBasedTranslator()
    for p in PROMPTS:
        cfg = t.translate(p)
        assert isinstance(cfg, GameConfig)
        assert len(cfg.paylines) >= 1
        for line in cfg.paylines:
            assert len(line) == cfg.game.grid.reels
            assert all(0 <= row < cfg.game.grid.rows for row in line)


def test_health():
    assert client.get("/health").json() == {"status": "ok"}


def test_translate_endpoint_extracts_params():
    r = client.post(
        "/translate",
        json={"prompt": "5x3 slot, 20 lines, 96% rtp, high volatility, wild scatter"},
    )
    assert r.status_code == 200, r.text
    body = r.json()
    assert body["config"]["game"]["grid"] == {"reels": 5, "rows": 3}
    assert len(body["config"]["paylines"]) == 20
    assert body["targets"]["rtp"] == 0.96
    assert body["targets"]["volatility"] == "high"


def test_validate_rejects_bad_config():
    r = client.post("/validate", json={"game": {"name": "x"}})
    assert r.status_code == 200
    assert r.json()["valid"] is False
    assert len(r.json()["errors"]) > 0


def test_simulate_endpoint():
    cfg = RuleBasedTranslator().translate("5x3, 10 lines, wild scatter").model_dump()
    r = client.post("/simulate", json={"config": cfg, "spins": 200000})
    assert r.status_code == 200, r.text
    body = r.json()
    assert 0.0 < body["exact_rtp"] < 10.0
    assert body["spins"] == 200000


def test_design_pipeline_hits_target():
    # no volatility word -> RTP-only tune (tight + fast)
    r = client.post(
        "/design",
        json={"prompt": "5x3 slot, 10 lines, 96% rtp, wild scatter", "spins": 200000, "generations": 100},
    )
    assert r.status_code == 200, r.text
    d = r.json()
    assert d["resolved_target_rtp"] == 0.96
    assert abs(d["tuned"]["after"]["exact_rtp"] - 0.96) < 0.02
