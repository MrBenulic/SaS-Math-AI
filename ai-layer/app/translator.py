"""AI Context Translator (ADR-008).

The LLM is OFF the critical path and NEVER generates code or math — it only extracts a small,
constrained *intent* object (grid size, line count, RTP/volatility target, which fixed mechanics to
include). Our deterministic code (`mechanics.build_base_config`) then composes the validated
GameConfig from the FIXED library, and the result passes the SAME Pydantic/schema validation as the
form. So a hallucinating model can at worst produce a wrong-but-valid game — never unsafe code or
corrupted math. Two providers ship: a deterministic rule-based default (no key) and a Groq adapter.
"""
from __future__ import annotations

import json
import os
import re
from pathlib import Path
from typing import Optional, Protocol

import httpx
from pydantic import BaseModel, Field

from .mechanics import VOLATILITY_PRESETS, build_base_config
from .models import GameConfig, LineEval, Targets


class Translator(Protocol):
    def translate(self, prompt: str) -> GameConfig: ...


# ---- Constrained intent: the ONLY thing the LLM is allowed to emit ----
class GameIntent(BaseModel):
    """Parameters the LLM may set. Everything is clamped/validated before building the config."""

    reels: int = 5
    rows: int = 3
    num_lines: int = 10
    rtp: Optional[float] = None  # fraction (0.96); percent (96) is normalized in build_from_intent
    volatility: Optional[str] = None  # must be a VOLATILITY_PRESETS key, else dropped
    has_wild: bool = True
    has_scatter: bool = True
    both_ways: bool = False
    name: Optional[str] = Field(default=None, max_length=60)


def build_from_intent(intent: GameIntent, fallback_name: str) -> GameConfig:
    """Build a validated GameConfig from a (possibly LLM-produced) intent — all values clamped here."""
    reels = min(max(intent.reels, 1), 16)
    rows = min(max(intent.rows, 1), 8)
    num_lines = min(max(intent.num_lines, 1), 200)
    volatility = intent.volatility if intent.volatility in VOLATILITY_PRESETS else None
    rtp = intent.rtp
    if rtp is not None and rtp > 1.5:  # the model gave a percentage (e.g. 96) — normalize
        rtp = rtp / 100.0
    line_eval = LineEval.both_ways if intent.both_ways else LineEval.left_to_right

    cfg = build_base_config(
        name=(intent.name or fallback_name or "AI slot")[:60],
        reels=reels,
        rows=rows,
        num_lines=num_lines,
        has_wild=intent.has_wild,
        has_scatter=intent.has_scatter,
        line_eval=line_eval,
    )
    cfg.targets = Targets(rtp=rtp, volatility=volatility)
    return cfg


class RuleBasedTranslator:
    """Keyword/regex extraction -> a validated GameConfig. The provider-agnostic default (no key)."""

    def translate(self, prompt: str) -> GameConfig:
        p = prompt.lower()

        reels, rows = 5, 3
        m = re.search(r"(\d+)\s*[x×]\s*(\d+)", p)
        if m:
            reels, rows = int(m.group(1)), int(m.group(2))

        num_lines = 10
        m = re.search(r"(\d+)\s*(?:pay)?lines?", p)
        if m:
            num_lines = int(m.group(1))

        rtp = None
        if "rtp" in p or "return" in p:
            m = re.search(r"(\d+(?:\.\d+)?)\s*%", p)
            if m:
                rtp = float(m.group(1)) / 100.0

        volatility = next((v for v in ["medium-high", "extreme", "high", "medium", "low"] if v in p), None)
        both_ways = "both ways" in p or "win both ways" in p

        has_wild = "no wild" not in p and "without wild" not in p
        has_scatter = "no scatter" not in p and "without scatter" not in p
        if "scatter" in p:
            has_scatter = True
        if "wild" in p:
            has_wild = True

        intent = GameIntent(
            reels=reels, rows=rows, num_lines=num_lines, rtp=rtp, volatility=volatility,
            has_wild=has_wild, has_scatter=has_scatter, both_ways=both_ways,
        )
        return build_from_intent(intent, fallback_name=prompt.strip())


# ---- Groq LLM adapter (real provider) ----
_GROQ_URL = "https://api.groq.com/openai/v1/chat/completions"
_DEFAULT_MODEL = "llama-3.3-70b-versatile"

_SYSTEM_PROMPT = (
    "You convert a casino slot-game description into a STRICT JSON object of PARAMETERS for a fixed "
    "math engine. You NEVER write code, never invent fields, never produce math. Output ONLY a JSON "
    "object with exactly these keys: reels (int 1-16), rows (int 1-8), num_lines (int 1-200), "
    "rtp (number 0-1 as a fraction, or null), volatility (one of: low, medium, medium-high, high, "
    "extreme, or null), has_wild (bool), has_scatter (bool), both_ways (bool), name (short string). "
    "Map '96% RTP' to rtp 0.96. Pick the closest volatility preset. If something is unspecified use "
    "sensible slot defaults: 5 reels, 3 rows, 20 lines, wild and scatter included."
)


def _load_groq_key() -> Optional[str]:
    """Key from the environment, falling back to ai-layer/.env (never logged, never committed)."""
    key = os.environ.get("GROQ_API_KEY")
    if key:
        return key.strip()
    env = Path(__file__).resolve().parents[1] / ".env"
    if env.exists():
        for line in env.read_text().splitlines():
            if line.startswith("GROQ_API_KEY="):
                return line.split("=", 1)[1].strip()
    return None


class GroqTranslator:
    """Real LLM adapter: Groq emits a constrained GameIntent (JSON mode); we build the config."""

    def __init__(self, model: Optional[str] = None):
        self.model = model or os.environ.get("GROQ_MODEL", _DEFAULT_MODEL)

    def translate(self, prompt: str) -> GameConfig:
        key = _load_groq_key()
        if not key:
            raise NotImplementedError(
                "Groq provider needs GROQ_API_KEY (set the env var or ai-layer/.env)."
            )
        payload = {
            "model": self.model,
            "messages": [
                {"role": "system", "content": _SYSTEM_PROMPT},
                {"role": "user", "content": prompt},
            ],
            "response_format": {"type": "json_object"},
            "temperature": 0.2,
        }
        try:
            resp = httpx.post(
                _GROQ_URL,
                headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
                json=payload,
                timeout=30.0,
            )
            resp.raise_for_status()
            content = resp.json()["choices"][0]["message"]["content"]
        except httpx.HTTPStatusError as e:
            raise RuntimeError(f"Groq API error {e.response.status_code}") from e
        except httpx.HTTPError as e:
            raise RuntimeError(f"Groq request failed: {e}") from e

        try:
            data = json.loads(content)
        except json.JSONDecodeError as e:
            raise RuntimeError("Groq did not return valid JSON") from e

        intent = GameIntent(**data)  # constrained: extra/garbage fields are ignored, types validated
        return build_from_intent(intent, fallback_name=prompt.strip())


def get_translator(provider: str = "rule") -> Translator:
    if provider in ("rule", "", None):
        return RuleBasedTranslator()
    if provider == "groq":
        return GroqTranslator()
    raise NotImplementedError(f"Unknown translator provider '{provider}' (use 'rule' or 'groq').")
