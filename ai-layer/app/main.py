"""FastAPI orchestration (M3). Pipeline: prompt -> validated JSON -> simulate -> tune.

The math is always computed by the Rust engine (ADR-001/008); this layer only translates, validates,
and orchestrates. Run: `uvicorn app.main:app --reload` (from ai-layer/, with the venv active).
"""
from __future__ import annotations

from typing import Any, Optional

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, ValidationError

from . import engine as eng
from .mechanics import VOLATILITY_PRESETS
from .models import GameConfig
from .translator import get_translator

app = FastAPI(title="SaS-Math AI — ai-layer", version="0.4.0")


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


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


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
