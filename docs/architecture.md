# Architecture

## System overview

SaS-Math AI is a pipeline that turns a designer's intent into certified slot math:

```
[1] Frontend UI ──▶ [2] AI Context Translator ──▶  game-config.json (validated)
                                                         │
                                                         ▼
                          [3] Core Math Engine  ◀──▶ [4] Auto-Tuner / Solver
                          (exact + Monte Carlo)        (hits RTP/vol/hit-freq…)
                                                         │
                                                         ▼
                                       [5] Analytics & Compliance (GLI/BMM export)
```

The **game config JSON** is the contract that flows through the whole system. It is defined once
in `schema/game-config.schema.json` and is the single source of truth for every module
(see ADR-004).

## Module responsibilities

### 1. Frontend UI  *(Next.js · Tailwind · Zustand · Monaco)* — M3
The designer sets parameters via UI toggles **and** a natural-language prompt. The UI form is the
**primary** input and is generated from the schema; Monaco exposes the raw JSON for power users.
Hard boundaries (e.g. "max win = 5000×") are locked in the UI so the solver always has a feasible
target.

### 2. AI Context Translator  *(Python · FastAPI · Pydantic · LLM)* — M3
Bridges human language → strict JSON. **The LLM never writes code** — it selects and parameterizes
mechanics from a fixed library and emits JSON via constrained decoding, which then passes through
the *same* Pydantic/schema validation as the form (ADR-008). The LLM is **off the critical path**:
the system is fully usable through the form alone. LLM provider is **undecided** (provider-agnostic
interface).

### 3. Core Math Engine  *(Rust · Rayon)* — M1
Two cooperating engines on one config:
- **Exact enumeration / convolution** — the certification source of truth. For a 5×3 lines game the
  reel-stop space (~32⁵) is fully enumerable → exact theoretical RTP to many decimals (ADR-001).
- **Deterministic Monte Carlo** — validates the harness against the exact math and covers features
  too complex to enumerate (cascades, hold-and-spin). Millions–billions of spins.

Foundations that make it certifiable: integer/`u128` accounting (ADR-002), counter-based namespaced
RNG (ADR-003), streaming integer statistics (ADR-012), max-win cap inside the math (ADR-005). The
engine is a **pure Rust crate** with no Python/FFI in its core (ADR-007).

### 4. Auto-Tuner / Solver  *(Rust prod · Python proto, via PyO3)* — M2
Adjusts symbol weights / paytables until the simulation hits target RTP, volatility, hit-frequency,
and bonus-frequency. Optimizes against the **exact engine** where possible (zero variance, fast),
using Common Random Numbers for the MC parts. Starts with **CMA-ES + penalty constraints**;
escalates to NSGA-II only for genuine trade-off fronts. Anti-degeneracy via **gameplay-shape**
hard constraints and an **RTP-contribution-by-feature** decomposition that must reconcile to the
unit (ADR-006). Calls the Rust core in-process via PyO3 (ADR-007).

### 5. Analytics & Compliance  *(ECharts/D3 · PDF/Excel)* — M4
Renders results and exports **GLI/BMM certification-ready** math sheets: combinatorics, exact
hit-frequencies per payline, reel-strip breakdowns, RTP-by-feature, and every MC metric **± a
confidence interval**. Export-boundary floats follow strict discipline (ADR-009).

## Why this shape

- **Exact-first** makes the product *certifiable*; Monte Carlo alone cannot certify high-volatility
  math (the original "1B spins < 0.05%" gate is statistically unreachable — see `parity-test.md`).
- **Determinism + namespaced RNG** make every result reproducible and **stable under future code
  changes**, so certification artifacts never silently invalidate.
- **One schema, three targets** removes the most dangerous class of bug: silent
  backend↔engine config drift.
- **LLM off the critical path** keeps a non-deterministic component out of the chain that produces
  certified numbers.

Full rationale and consequences: [`design-decisions.md`](design-decisions.md).
