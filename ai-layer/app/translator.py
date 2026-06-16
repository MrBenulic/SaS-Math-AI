"""AI Context Translator (ADR-008).

The LLM is OFF the critical path and NEVER generates code — it only selects and parameterizes
mechanics from the fixed library (`mechanics.py`), and its output passes the SAME Pydantic/schema
validation as the form. Here we ship a deterministic rule-based translator (no API key needed); a
real LLM adapter (Claude/OpenAI) is a drop-in that would use constrained decoding against the schema.
"""
from __future__ import annotations

import re
from typing import Protocol

from .mechanics import VOLATILITY_PRESETS, build_base_config
from .models import GameConfig, LineEval, Targets


class Translator(Protocol):
    def translate(self, prompt: str) -> GameConfig: ...


class RuleBasedTranslator:
    """Keyword/regex extraction -> a validated GameConfig. The provider-agnostic default."""

    def translate(self, prompt: str) -> GameConfig:
        p = prompt.lower()

        reels, rows = 5, 3
        m = re.search(r"(\d+)\s*[x×]\s*(\d+)", p)
        if m:
            reels, rows = int(m.group(1)), int(m.group(2))
            reels = min(max(reels, 1), 16)
            rows = min(max(rows, 1), 8)

        num_lines = 10
        m = re.search(r"(\d+)\s*(?:pay)?lines?", p)
        if m:
            num_lines = min(max(int(m.group(1)), 1), 200)

        rtp = None
        if "rtp" in p or "return" in p:
            m = re.search(r"(\d+(?:\.\d+)?)\s*%", p)
            if m:
                rtp = float(m.group(1)) / 100.0

        volatility = next((v for v in ["medium-high", "extreme", "high", "medium", "low"] if v in p), None)
        line_eval = LineEval.both_ways if ("both ways" in p or "win both ways" in p) else LineEval.left_to_right

        # Default to including a wild + scatter unless explicitly excluded.
        has_wild = "no wild" not in p and "without wild" not in p
        has_scatter = "no scatter" not in p and "without scatter" not in p
        if "scatter" in p:
            has_scatter = True
        if "wild" in p:
            has_wild = True

        name = (prompt.strip()[:60] or "Untitled slot")
        cfg = build_base_config(
            name=name,
            reels=reels,
            rows=rows,
            num_lines=num_lines,
            has_wild=has_wild,
            has_scatter=has_scatter,
            line_eval=line_eval,
        )
        cfg.targets = Targets(rtp=rtp, volatility=volatility)
        return cfg


class ProviderTranslator:
    """Stub for a real LLM adapter. Wire a provider + API key, then constrain decoding to the schema."""

    def __init__(self, provider: str):
        self.provider = provider

    def translate(self, prompt: str) -> GameConfig:
        raise NotImplementedError(
            f"LLM provider '{self.provider}' is not configured. Set an API key and implement "
            "constrained decoding against schema/game-config.schema.json + the fixed mechanics "
            "library. (Provider choice is deferred — see docs/design-decisions.md ADR-008.)"
        )


def get_translator(provider: str = "rule") -> Translator:
    return RuleBasedTranslator() if provider == "rule" else ProviderTranslator(provider)
