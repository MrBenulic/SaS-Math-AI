"""FastAPI orchestration (M3). Pipeline: prompt -> validated JSON -> simulate -> tune.

The math is always computed by the Rust engine (ADR-001/008); this layer only translates, validates,
and orchestrates. Run: `uvicorn app.main:app --reload` (from ai-layer/, with the venv active).
"""
from __future__ import annotations

from typing import Any, Optional

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, ValidationError

from . import engine as eng
from .artifact import build_published_artifact, config_hash
from .binding import EVENT_TYPES, PresentationBind, validate_binding
from .mechanics import VOLATILITY_PRESETS
from .models import GameConfig
from .translator import get_translator

app = FastAPI(title="SaS-Math AI — ai-layer", version="0.4.0")
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "http://127.0.0.1:3000"],
    allow_methods=["*"],
    allow_headers=["*"],
)


class TranslateReq(BaseModel):
    prompt: str
    provider: str = "rule"


class SimulateReq(BaseModel):
    config: GameConfig
    spins: int = 2_000_000


class TuneReq(BaseModel):
    config: GameConfig
    target_rtp: float
    generations: int = 200
    target_volatility: Optional[float] = None
    target_hit_frequency: Optional[float] = None


class DesignReq(BaseModel):
    prompt: str
    spins: int = 1_000_000
    generations: int = 150


class BindValidateReq(BaseModel):
    config: GameConfig
    binding: PresentationBind


class PublishReq(BaseModel):
    config: GameConfig
    spins: int = 2_000_000


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/bind/event-types")
def bind_event_types() -> dict[str, Any]:
    """The closed set of engine event types a presentation binding must cover (the art↔math contract)."""
    return {"event_types": list(EVENT_TYPES)}


@app.post("/validate-bind")
def validate_bind(r: BindValidateReq) -> dict[str, Any]:
    """Bidirectional completeness check: every symbol + every event type is bound to art (the wedge)."""
    return validate_binding(r.config, r.binding).model_dump()


@app.post("/config-hash")
def config_hash_endpoint(config: GameConfig) -> dict[str, str]:
    """The SHA-256 certification fingerprint of a validated config (stable, reproducible)."""
    return {"config_hash": config_hash(config)}


@app.post("/publish")
def publish(r: PublishReq) -> dict[str, Any]:
    """Build a reproducible published-math artifact: fingerprint + canonical config + engine results."""
    try:
        results = eng.simulate(r.config.model_dump(), r.spins)
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))
    return build_published_artifact(r.config, results)


@app.post("/translate")
def translate(r: TranslateReq) -> dict[str, Any]:
    try:
        cfg = get_translator(r.provider).translate(r.prompt)
    except NotImplementedError as e:
        raise HTTPException(status_code=501, detail=str(e))
    return {"config": cfg.model_dump(), "targets": cfg.targets.model_dump() if cfg.targets else None}


@app.post("/validate")
def validate(payload: dict[str, Any]) -> dict[str, Any]:
    try:
        GameConfig(**payload)
        return {"valid": True, "errors": []}
    except ValidationError as e:
        return {"valid": False, "errors": e.errors()}


@app.post("/simulate")
def simulate(r: SimulateReq) -> dict[str, Any]:
    try:
        return eng.simulate(r.config.model_dump(), r.spins)
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.post("/tune")
def tune(r: TuneReq) -> dict[str, Any]:
    try:
        return eng.tune(
            r.config.model_dump(),
            r.target_rtp,
            r.generations,
            r.target_volatility,
            r.target_hit_frequency,
        )
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))


@app.post("/design")
def design(r: DesignReq) -> dict[str, Any]:
    """Full pipeline for the M3 gate: prompt -> config -> simulate -> tune to the prompt's targets."""
    cfg = get_translator("rule").translate(r.prompt)
    config = cfg.model_dump()
    target_rtp = cfg.targets.rtp if (cfg.targets and cfg.targets.rtp) else 0.96
    target_vol = (
        VOLATILITY_PRESETS.get(cfg.targets.volatility)
        if (cfg.targets and cfg.targets.volatility)
        else None
    )
    try:
        before = eng.simulate(config, r.spins)
        tuned = eng.tune(config, target_rtp, r.generations, target_vol, None)
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))
    return {
        "prompt": r.prompt,
        "config": config,
        "targets": cfg.targets.model_dump() if cfg.targets else None,
        "resolved_target_rtp": target_rtp,
        "resolved_target_volatility": target_vol,
        "before": before,
        "tuned": tuned,
    }
