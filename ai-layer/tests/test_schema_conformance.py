"""ADR-004 schema-conformance: the JSON schema is the single source of truth for the config consumed
by TS (frontend), Pydantic (ai-layer), AND the Rust engine (serde). These tests catch the dangerous
drift — a config the engine would execute but that no longer matches the schema, or vice versa — by
validating real example configs and the Pydantic/translator output against the schema itself."""
from __future__ import annotations

import json
from pathlib import Path

import jsonschema
import pytest

from app.mechanics import build_base_config
from app.models import GameConfig
from app.translator import get_translator

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = json.loads((ROOT / "schema" / "game-config.schema.json").read_text())
EXAMPLES = sorted((ROOT / "core-math" / "examples").glob("*.json"))


def test_examples_exist():
    assert EXAMPLES, "no example configs found under core-math/examples"


@pytest.mark.parametrize("path", EXAMPLES, ids=lambda p: p.name)
def test_example_conforms_to_schema(path: Path):
    jsonschema.validate(json.loads(path.read_text()), SCHEMA)  # raises on drift


@pytest.mark.parametrize("path", EXAMPLES, ids=lambda p: p.name)
def test_example_parses_with_pydantic(path: Path):
    # schema-shaped JSON must load into the Pydantic mirror (ADR-004 backend target)
    GameConfig(**json.loads(path.read_text()))


def test_built_config_conforms_to_schema():
    cfg = build_base_config(name="conformance", reels=5, rows=3, num_lines=20)
    jsonschema.validate(cfg.model_dump(mode="json", exclude_none=True), SCHEMA)


def test_translator_output_conforms_to_schema():
    cfg = get_translator("rule").translate("5x3, 20 lines, 96% rtp, high volatility, wild and scatter")
    jsonschema.validate(cfg.model_dump(mode="json", exclude_none=True), SCHEMA)
