# Roadmap

SemVer map and milestone gates. Each milestone ends with a **stable** release on `main`.

| Version | Milestone | Theme |
|---------|-----------|-------|
| `v0.1.0` | **Foundation** ✅ | repo, versioning, design docs, schema draft, tooling |
| `v0.2.0` | **M1** | Core Math Engine (5×3 lines) |
| `v0.3.0` | **M2** | Auto-Tuner / Solver |
| `v0.4.0` | **M3** | AI translator + FastAPI + frontend |
| `v1.0.0` | **M4** | Advanced mechanics + GLI/BMM exporter (commercial prototype) |

Beta snapshots during a milestone are tagged `vX.Y.Z-beta.N` (pre-releases).

---

## v0.1.0 — Foundation ✅ (this iteration)
Project home, private repo, stable/beta branching, feature/metrics workbook, `reboot.md`, design
docs (ADRs), draft schema, plugins, Rust toolchain. **No engine code.**
**Gate:** clean documented checkpoint; everything in `README.md` "Verification" passes.

## v0.2.0 — M1: Core Math Engine
- Counter-based **namespaced RNG** (ADR-003) + integer/`u128` accounting (ADR-002).
- **Exact enumeration RTP** (ADR-001) + line-evaluation "Lego blocks" (LTR / ways / both-ways,
  scatter/wild rules) unit-tested against known sheets.
- Deterministic **Monte Carlo** (10M spins) + streaming integer stats (ADR-012).
- Minimal **numeric math-sheet** output (CSV/JSON) + CLI.
- **Gate (the big one):** the **reframed Math Parity Test** (`parity-test.md`) passes against a
  known certified game's exact math. Also: seed-replay test (50 runs, identical to the unit);
  10M-spin MC target < 2s. *Correctness of foundation gates performance.*

## v0.3.0 — M2: Auto-Tuner / Solver
- Optimize against the exact engine; CRN for MC parts (ADR-006); CMA-ES + penalty constraints.
- Weights→strip realization + exact re-verify; gameplay-shape hard constraints; RTP-by-feature.
- Rust core exposed to Python via PyO3 (ADR-007).
- **Gate:** feed a broken config (e.g. 140% RTP) → solver returns a fixed, *playable* 96% config
  (hit/bonus frequency realistic, not degenerate).

## v0.4.0 — M3: AI + Frontend
- AI Context Translator (provider-agnostic, constrained decoding, off critical path — ADR-008) +
  FastAPI orchestration.
- Next.js + Tailwind + Zustand + Monaco; schema-generated form (ADR-004).
- Playwright E2E; Postgres/SQLite persistence for configs & results.
- **Gate:** a user types a prompt → JSON → simulate → tune, end-to-end in the browser.

## v1.0.0 — M4: Advanced mechanics + Compliance
- Cascades, expanding wilds, multipliers (MC / Markov absorbing-state for non-enumerable parts);
  declared recursion caps (ADR-011).
- Full **GLI/BMM PDF/Excel exporter** with combinatorics, per-payline hit-freqs, reel breakdowns,
  RTP-by-feature, MC metrics ± CI (ADR-009).
- **Gate:** exported math sheet matches a certified live game's sheet side-by-side.

---

## Cross-cutting "definition of done" per milestone
1. Tests green (unit + the milestone gate). 2. `docs/` + `reboot.md` updated. 3. Excel workbook
metrics updated. 4. Merge `beta` → `main`, tag stable, cut a GitHub Release. 5. Tag a beta
pre-release for any in-progress follow-up.
