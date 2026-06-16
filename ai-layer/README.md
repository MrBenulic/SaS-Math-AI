# ai-layer (Python · FastAPI) — Module 2

The AI Context Translator + API orchestration. **M3-beta.1 implemented** (backend pipeline). The
Next.js UI is the next slice.

## Principles
- **The LLM never writes code (ADR-008).** It composes validated JSON from the fixed `mechanics.py`
  library, and its output passes the *same* Pydantic/schema validation as the form.
- **Off the critical path:** fully usable without an LLM — a deterministic rule-based translator ships
  by default (no API key needed).
- **Provider-agnostic:** `Translator` interface with a `RuleBasedTranslator` and a `ProviderTranslator`
  stub for a real LLM (Claude/OpenAI) — wire a key later and constrain decoding to the schema.

## Layout
```
ai-layer/
├─ pyproject.toml
├─ app/
│  ├─ main.py         FastAPI: /translate /validate /simulate /tune /design /health
│  ├─ models.py       Pydantic GameConfig (mirrors schema/game-config.schema.json; ADR-004)
│  ├─ mechanics.py    the fixed mechanics library + base-config builder
│  ├─ translator.py   Translator interface + RuleBasedTranslator + provider stub
│  └─ engine.py       bridge to the Rust `engine` binary (JSON over stdin/stdout; PyO3 is the upgrade)
└─ tests/test_pipeline.py
```

## Run
```bash
python3 -m venv .venv && .venv/bin/pip install -e '.[dev]'
(cd ../core-math && cargo build --release --bin engine)   # the math engine the API calls
.venv/bin/uvicorn app.main:app --reload                   # http://127.0.0.1:8000/docs
.venv/bin/python -m pytest -q                              # 6 tests
```

## The pipeline (M3 gate)
`POST /design {"prompt": "..."}` runs **prompt → validated JSON → simulate → tune** and returns the
before/after metrics + tuned config. Example: *"5x3 slot, 20 paylines, 96% RTP, medium volatility,
wild and scatter"* → tuned to 96.00% RTP (realize-delta ~0.0001). The Rust engine/solver do all the
math; this layer only translates, validates, and orchestrates.

> Bridge note: `engine.py` shells out to `core-math/target/release/engine`. Override with
> `SAS_ENGINE_BIN`. PyO3 in-process bindings (ADR-007) are the planned production upgrade.
