# SaS-Math AI

**AI-Driven Slot Math Generation & Tuning Engine** — a B2B SaaS platform that automates the
creation, simulation, and tuning of slot-machine mathematical models. It combines an LLM
"context translator" with a high-speed, **deterministic** Monte Carlo **+ exact-combinatorial**
math engine, an auto-tuning solver, and a GLI/BMM certification-ready exporter.

> **Status:** `v0.1.0` — Foundation (design + scaffolding). _No engine code yet._
> Live context & handoff: [`reboot.md`](reboot.md) · What's next: [`docs/roadmap.md`](docs/roadmap.md)

---

## What it does

A game designer describes a slot game in plain language and UI toggles. The system then:

1. **Translates** that into a strict, validated JSON **game config**. The LLM only *composes*
   from a fixed library of pre-defined mechanics ("Lego blocks") — **it never writes code**.
2. **Computes exact theoretical RTP** by full enumeration where feasible, and runs
   **deterministic Monte Carlo** for features too complex to enumerate.
3. **Auto-tunes** reel weights / paytables to hit target RTP, volatility, hit-frequency, and
   bonus-frequency — *without* degenerate "boring" math (e.g. dumping RTP into dead spins).
4. **Exports** GLI/BMM certification-ready math sheets (combinatorics, per-payline hit
   frequencies, reel-strip breakdowns).

## Architecture (5 modules)

| # | Module | Stack | Status |
|---|--------|-------|--------|
| 1 | Frontend UI | Next.js · Tailwind · Zustand · Monaco | planned (M3) |
| 2 | AI Context Translator | Python · FastAPI · Pydantic · LLM *(provider-agnostic)* | planned (M3) |
| 3 | Core Math Simulator | Rust · Rayon | planned (M1) |
| 4 | Auto-Tuner / Solver | Rust *(+ Python proto)* · CMA-ES / NSGA-II | planned (M2) |
| 5 | Analytics & Compliance | ECharts / D3 · PDF / Excel export | planned (M4) |

Details: [`docs/architecture.md`](docs/architecture.md) · Locked engineering decisions:
[`docs/design-decisions.md`](docs/design-decisions.md)

## Repository layout

```
SaS-Math AI app/
├─ README.md            this file
├─ reboot.md            live context-handoff doc (kept up to date every session)
├─ CLAUDE.md            project rules for AI-assisted development
├─ docs/
│  ├─ architecture.md      the 5 modules and how they connect
│  ├─ design-decisions.md  ADRs — the load-bearing engineering decisions
│  ├─ roadmap.md           milestones, gates, and SemVer map
│  ├─ versioning.md        stable/beta branching, releases, rollback
│  └─ parity-test.md       the (reframed) Math Parity Test methodology
├─ schema/
│  └─ game-config.schema.json   v0.1 DRAFT — single source of truth (tri-target contract)
├─ core-math/           Rust engine            (built in M1)
├─ ai-layer/            Python FastAPI + LLM    (built in M3)
├─ frontend/            Next.js app             (built in M3)
└─ tools/               codegen + parity harness
```

A separate **`SaS-Math-AI_Features_and_Metrics.xlsx`** workbook (feature catalogue + metric
targets) lives in the project root.

## Versioning & rollback

Stable on **`main`** (tags `vX.Y.Z`), active work on **`beta`** (tags `vX.Y.Z-beta.N`). Any
checkpoint is one `git checkout <tag>` away, and every stable tag is a downloadable GitHub
Release. Full policy: [`docs/versioning.md`](docs/versioning.md).

## Roadmap

`v0.1.0` Foundation → `v0.2.0` M1 engine → `v0.3.0` M2 solver → `v0.4.0` M3 AI+frontend →
`v1.0.0` M4 compliance prototype. See [`docs/roadmap.md`](docs/roadmap.md).

## License

Proprietary and confidential — see [`LICENSE`](LICENSE).
