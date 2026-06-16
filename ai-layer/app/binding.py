"""Presentation binding — the art↔math bridge.

PRESENTATION ONLY: this layer lives entirely outside the certified-math core (core-math), so the
certification artifact stays pure (ADR-001). It maps the math's stable integer symbol ids and the
engine's closed set of event types to concrete art assets (Spine skeletons, atlas regions,
spritesheet frames, sounds). The bidirectional completeness check (`validate_binding`) — every
symbol AND every event type must be bound, and nothing may reference an id/event that doesn't exist —
is what makes "art + certified math = one verifiable package" a hard guarantee.

Mirrors schema/presentation-bind.schema.json (ADR-004 single-source-of-truth discipline).
"""
from __future__ import annotations

from typing import Optional

from pydantic import BaseModel, Field

from .models import GameConfig

# Closed, versioned set of event types the engine's RoundOutcome can drive. A binding must cover all
# of these. Mirrors core-math's RoundOutcome (line_win / scatter_win / cap_applied) plus the spin
# lifecycle a renderer needs. Keep in sync with schema/presentation-bind.schema.json.
EVENT_TYPES: tuple[str, ...] = (
    "spin_start",
    "reels_settled",
    "line_win",
    "scatter_win",
    "cap_applied",
    "spin_end",
)


class SpriteSheet(BaseModel):
    sheet: str
    frames: list[str] = Field(default_factory=list)
    fps: int = Field(default=30, gt=0)


class SymbolBinding(BaseModel):
    """Art for one math symbol id. Must resolve to at least one renderable (skeleton/region/sheet)."""
    symbol_id: int = Field(ge=0)
    skeleton: Optional[str] = None
    atlas: Optional[str] = None
    default_skin: Optional[str] = None
    idle_anim: Optional[str] = None
    land_anim: Optional[str] = None
    win_anim: Optional[str] = None
    atlas_region: Optional[str] = None
    spritesheet: Optional[SpriteSheet] = None
    sound_cue: Optional[str] = None

    def has_renderable(self) -> bool:
        return bool(self.skeleton or self.atlas_region or self.spritesheet)


class EventBinding(BaseModel):
    event_type: str
    anim: Optional[str] = None
    target: Optional[str] = None
    sound: Optional[str] = None
    blocking: bool = False
    min_duration_ms: int = Field(default=0, ge=0)


class PresentationBind(BaseModel):
    bind_version: str = "0.1.0"
    # Optional content-hash of the math config this binding targets (filled when configs are hashed).
    targets_config_hash: Optional[str] = None
    symbols: list[SymbolBinding] = Field(default_factory=list)
    events: list[EventBinding] = Field(default_factory=list)


class BindReport(BaseModel):
    valid: bool
    missing_symbols: list[int] = Field(default_factory=list)
    unknown_symbols: list[int] = Field(default_factory=list)
    unrenderable_symbols: list[int] = Field(default_factory=list)
    missing_events: list[str] = Field(default_factory=list)
    unknown_events: list[str] = Field(default_factory=list)
    errors: list[str] = Field(default_factory=list)


def validate_binding(config: GameConfig, binding: PresentationBind) -> BindReport:
    """Bidirectional completeness check between a math config and an art binding."""
    config_symbol_ids = {s.id for s in config.symbols}

    bound_by_id: dict[int, SymbolBinding] = {}
    unknown_symbols: list[int] = []
    for sb in binding.symbols:
        if sb.symbol_id not in config_symbol_ids:
            unknown_symbols.append(sb.symbol_id)
        else:
            bound_by_id[sb.symbol_id] = sb

    missing_symbols = sorted(config_symbol_ids - set(bound_by_id))
    unrenderable_symbols = sorted(sid for sid, sb in bound_by_id.items() if not sb.has_renderable())

    bound_events = {e.event_type for e in binding.events}
    missing_events = [et for et in EVENT_TYPES if et not in bound_events]
    unknown_events = sorted(bound_events - set(EVENT_TYPES))

    errors: list[str] = []
    if missing_symbols:
        errors.append(f"symbols with no art binding: {missing_symbols}")
    if unrenderable_symbols:
        errors.append(
            f"symbols bound but with no renderable asset (skeleton/atlas_region/spritesheet): {unrenderable_symbols}"
        )
    if unknown_symbols:
        errors.append(f"bindings reference unknown symbol ids: {sorted(unknown_symbols)}")
    if missing_events:
        errors.append(f"event types with no binding: {missing_events}")
    if unknown_events:
        errors.append(f"bindings reference unknown event types: {unknown_events}")

    return BindReport(
        valid=not errors,
        missing_symbols=missing_symbols,
        unknown_symbols=sorted(unknown_symbols),
        unrenderable_symbols=unrenderable_symbols,
        missing_events=missing_events,
        unknown_events=unknown_events,
        errors=errors,
    )
