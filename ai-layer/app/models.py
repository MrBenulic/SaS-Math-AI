"""Pydantic models — the Python target of the tri-target schema (ADR-004).

Mirrors schema/game-config.schema.json. (Codegen via datamodel-code-generator is the planned
refinement; hand-written here for M3-beta.1.) Map keys (`pays`, `weights`) are strings to match the
Rust `serde` config and keep JSON round-trips robust.
"""
from __future__ import annotations

from enum import Enum
from typing import Literal, Optional

from pydantic import BaseModel, Field


class SymbolKind(str, Enum):
    regular = "regular"
    wild = "wild"
    scatter = "scatter"


class Symbol(BaseModel):
    id: int = Field(ge=0)
    name: str
    kind: SymbolKind
    substitutes_for: list[int] = Field(default_factory=list)


class Grid(BaseModel):
    reels: int = Field(ge=1, le=16)
    rows: int = Field(ge=1, le=8)


class Game(BaseModel):
    name: str
    grid: Grid
    bet_units: int = Field(ge=1)
    denomination_scale: int = 100


class Reels(BaseModel):
    strips: Optional[list[list[int]]] = None
    weights: Optional[list[dict[str, int]]] = None


class PaytableEntry(BaseModel):
    symbol_id: int = Field(ge=0)
    pays: dict[str, int]


class MaxWin(BaseModel):
    cap_x: float = Field(gt=0)


class LineEval(str, Enum):
    left_to_right = "left_to_right"
    both_ways = "both_ways"
    ways_243 = "ways_243"


class RngConfig(BaseModel):
    generator: Literal["chacha"] = "chacha"  # only ChaCha8 is implemented (see engine rng.rs)
    master_seed: int = Field(default=0, ge=0)


class Targets(BaseModel):
    """Advisory tuning targets (consumed by the solver, ignored by the math engine)."""
    rtp: Optional[float] = None
    volatility: Optional[str] = None  # preset name; mapped to a number when tuning
    hit_frequency: Optional[float] = None


class GameConfig(BaseModel):
    schema_version: str = "0.1.0"
    game: Game
    symbols: list[Symbol]
    reels: Reels
    paytable: list[PaytableEntry]
    paylines: list[list[int]]
    line_eval: LineEval = LineEval.left_to_right
    max_win: MaxWin
    rng: RngConfig = Field(default_factory=RngConfig)
    features: list = Field(default_factory=list)
    targets: Optional[Targets] = None
