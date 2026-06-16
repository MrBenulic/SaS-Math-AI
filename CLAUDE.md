# CLAUDE.md — SaS-Math AI

Project rules for AI-assisted development. These OVERRIDE default behavior.

## What this project is
A B2B SaaS **slot-math generation & tuning engine**. Correctness and reproducibility are
regulatory requirements (GLI/BMM certification), not nice-to-haves. When in doubt, choose the
option that is **deterministic, exactly reproducible, and auditable**.

## Non-negotiable engineering rules
1. **Never generate executable code in the math path.** The LLM layer may ONLY compose validated
   JSON from the fixed library of mechanics ("Lego blocks"). No dynamic code, no `eval`, no
   inventing mechanics on the fly. All math is computed by the Rust engine from validated JSON.
2. **Integer / fixed-point accounting only** for money/payouts. Track winnings in smallest units
   as integers; accumulators are `u128`. Render to decimal **only** at the output boundary.
   Never sum payouts in floating point.
3. **RNG is counter-based and namespaced.** Every random draw is a pure function of
   `(subsystem, spin_index, draw_index)`. Adding draws in one feature must never perturb another
   subsystem's bitstream. Determinism = same seed ⇒ identical result, across threads and runs.
4. **The schema is the single source of truth.** `schema/game-config.schema.json` generates the
   TypeScript types, the Pydantic models, AND the Rust `serde` structs. Change the schema, then
   regenerate — never hand-edit a generated type. Bump `schema_version` on any change.
5. **Old configs reproduce identical math forever.** Config files are certification artifacts.
   Engine changes must be version-pinned so a v1 game's numbers never silently change.

See `docs/design-decisions.md` for the full rationale (ADR-001 … ADR-00n).

## Workflow & versioning
- Work on **`beta`** (or `feature/*` off `beta`); promote to **`main`** only after validation.
- `main` = stable, tagged `vX.Y.Z`. `beta` snapshots tagged `vX.Y.Z-beta.N`. See `docs/versioning.md`.
- **Conventional Commits** (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).
- End every commit message with:
  `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`
- **Tag a stable checkpoint before any risky/large change** so rollback is always one step away.

## Every session
- Read `reboot.md` first; **update it at the end of the session** (status, done/next, open
  decisions) and keep its "fresh-chat prompt" current.
- Keep `SaS-Math-AI_Features_and_Metrics.xlsx` in sync as features/metrics change.

## Stack
Rust (core engine + production solver) · Python/FastAPI/Pydantic (AI layer + solver proto, via
PyO3) · Next.js/Tailwind/Zustand/Monaco (frontend). LLM provider: **undecided** — code against a
provider-agnostic interface until chosen.
