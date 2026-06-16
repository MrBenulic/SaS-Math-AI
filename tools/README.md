# tools

Build/dev tooling that ties the modules together. **Not yet implemented.**

## Planned
- **`gen-types/`** — regenerate all three targets from the one schema (ADR-004):
  - TypeScript via `json-schema-to-typescript` → `frontend/lib/types.ts`
  - Pydantic via `datamodel-code-generator` → `ai-layer/app/models.py`
  - Rust serde validation/derive check → `core-math/src/config.rs`
  - plus a **round-trip conformance test** (serialize from each target, assert semantic equality).
- **`parity-harness/`** — drive the Math Parity Test (`docs/parity-test.md`): load a certified
  game's params, run exact + MC, diff against the certified sheet.
- **`schema-migrate/`** — forward-migration functions per `schema_version` bump.

Wired up starting in M1 (gen-types + parity-harness).
