# ai-layer (Python · FastAPI) — Module 2

The AI Context Translator + API orchestration. **Not yet implemented** — built in **M3** (`v0.4.0`).

## Principles
- **The LLM never writes code.** It composes validated JSON from a fixed library of mechanics via
  constrained/structured decoding, then passes the *same* validation as the UI form (ADR-008).
- **Off the critical path:** the system is fully usable through the form alone.
- **Provider-agnostic:** LLM provider is undecided — code against an interface (`LlmTranslator`)
  with swappable backends; pick the model in M3.

## Planned layout
```
ai-layer/
├─ pyproject.toml
├─ app/
│  ├─ main.py            FastAPI app + routes (translate, validate, simulate, tune)
│  ├─ models.py          Pydantic models generated from schema/game-config.schema.json (ADR-004)
│  ├─ translator/        LlmTranslator interface + provider adapters (constrained decoding)
│  ├─ mechanics.py       the fixed enum library the LLM may select from
│  └─ engine.py          calls core-math via PyO3 (release GIL; config in / metrics out; ADR-007)
└─ tests/
   └─ fuzz_schema.py     feed N stylized prompts; assert 100% emit schema-valid JSON
```
