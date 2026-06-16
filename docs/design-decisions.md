# Design Decisions (ADRs)

Architecture Decision Records for SaS-Math AI. Each is a deliberate, hard-to-reverse choice that
the rest of the build depends on. Status: **Accepted** unless noted. Locked at `v0.1.0`.

> The five highest-risk "lock-before-M1" items are **ADR-001, 002, 003, 004, 005**. Getting any of
> them wrong forces a costly rebuild and/or silently invalidates certification artifacts.

---

## ADR-001 — Exact combinatorial math is the certification source of truth
**Context.** Certification labs (GLI/BMM) need *exact theoretical* RTP, not an estimate. A 1B-spin
Monte Carlo on a high-volatility game (max win 5000×) cannot reach <0.05% precision on RTP — and
bonus / max-win frequency won't converge at all — because per-spin σ dominates (SE ≈ σ/√N).
**Decision.** Compute **exact RTP by full enumeration / convolution** wherever the state space is
finite (5×3 lines ≈ 32⁵ ≈ 33M combinations — trivial). Use **Monte Carlo only** to (a) validate the
engine harness against the exact math and (b) cover features that are genuinely not enumerable
(cascades, hold-and-spin, symbol-collection metas). Build **both** engines and a routing rule.
**Consequences.** The Parity Test is reframed around exact math (see ADR-010 / `parity-test.md`).
The solver gets a fast, zero-variance objective (ADR-006). Enumeration must apply the max-win cap
*inside* the convolution (ADR-005).

## ADR-002 — Integer / fixed-point accounting end-to-end (`u128` accumulators)
**Context.** Floating-point summation is non-associative (parallel reduction order changes results)
**and** loses low-order bits as the accumulator grows over billions of spins. Both break exact
reproducibility. `u64` overflows past ~10B spins / high denominations / stacked multipliers.
**Decision.** Track all money in the smallest integer unit. **Per-spin payout in `u64` with
overflow guards; running totals and sums-of-squares in `u128`.** RTP is a rational
(`total_won / total_bet`) rendered to decimal **only** at the output boundary. Per-thread integer
accumulators merge in a deterministic, order-independent way.
**Consequences.** No float anywhere in the money path. Cascade/multiplier intermediate steps must
be range-checked. Variance via integer sum-of-squares, not Welford-in-f64.

## ADR-003 — Counter-based, namespaced RNG (the keystone)
**Context.** "Same seed ⇒ same result across threads" requires that a spin's randomness not depend
on thread scheduling or on other spins. The harder, unstated requirement is **stability under code
change**: adding a feature in M4 must not change base-game results for a given seed, or every prior
certification artifact silently invalidates.
**Decision.** Use a **counter-based generator (Philox/Threefry**, or ChaCha for a named crypto
primitive) — *not* PCG (a sequential stream generator). Every draw is a pure function of a
structured counter `(subsystem, spin_index, draw_index)`. Partition the RNG domain **per subsystem**
(base reels / features / bonus-picks / retriggers) so draws in one never shift another's bitstream.
Document the output→reel-stop mapping (rejection sampling, no modulo bias).
**Consequences.** Enables thread-independent determinism, stability under code change, and Common
Random Numbers for the solver (ADR-006). **Must be locked before any engine code.** PRNG period
must vastly exceed total lifetime draws (Philox/ChaCha: fine; never an LCG).

## ADR-004 — One schema, three generated targets, versioned from commit #1
**Context.** The config is consumed by the frontend (TS), the AI layer (Pydantic), **and executed
by the Rust engine (`serde`)**. The dangerous drift is backend↔engine: a silently-ignored field
corrupts the math.
**Decision.** `schema/game-config.schema.json` is the **single source of truth**. Generate TS
(`json-schema-to-typescript`), Pydantic (`datamodel-code-generator`), and validate the Rust `serde`
structs against it, with a round-trip conformance test across all three. Every config carries
`schema_version`; provide forward-migration functions. **Engine guarantees old configs reproduce
byte-identical math forever** (configs are legal/audit artifacts).
**Consequences.** Generated types are never hand-edited. Schema changes are deliberate, versioned
events.

## ADR-005 — Max-win cap is a first-class math constraint
**Context.** A hard cap (e.g. 5000×) is a non-smooth clamp on the payout function. Applied *after*
the fact it gives the wrong RTP; it also creates a plateau the solver can exploit (degenerate "RTP
via capped wins").
**Decision.** Model `min(win, cap)` **inside** enumeration/convolution and simulation — capped
probability mass redistributes and changes the *exact* RTP. Track **"% of RTP delivered at the cap"**
as a guard metric exposed to the solver and the math sheet.
**Consequences.** Cap is part of the math model, not a display filter. Solver objective is
cap-aware (ADR-006).

## ADR-006 — Solver: exact-objective + CRN, reduced dimensionality, shape constraints
**Context.** Slot math is highly non-linear ("butterfly effect"); naïve solvers find degenerate or
locally-optimal "boring" math.
**Decision.** (a) Optimize against the **exact engine** where available; use **Common Random
Numbers** for MC parts so candidate differences reflect parameter changes, not RNG noise (depends on
ADR-003 namespacing). (b) Reduce dimensionality by tuning **symbol weights**, then **realize a
concrete reel strip and re-verify exact RTP** (weights can't express positional/adjacency
constraints labs care about). (c) Start with **CMA-ES + penalty constraints**; escalate to NSGA-II
only for genuine trade-off fronts. (d) **Anti-degeneracy:** gameplay-shape targets (win-distribution
percentiles, median win) as *hard* constraints, plus an **RTP-contribution-by-feature** breakdown
that must reconcile to the unit. (e) Seed the solver run itself so results are reproducible/auditable.
**Status.** Accepted in principle; algorithm choice (CMA-ES vs NSGA-II) is swappable behind the
solver interface and finalized in M2.

## ADR-007 — Pure Rust core + thin PyO3 wrapper
**Context.** The Python solver prototype must call the engine without per-evaluation marshalling
overhead.
**Decision.** The engine is a **pure Rust crate** (also linked by the CLI and the native solver).
Python access is a **separate thin PyO3 crate**: pass config in / scalar metrics out (never per-spin
data), **release the GIL** around compute, and **`catch_unwind`** at the FFI boundary.
**Consequences.** No Python in the hot loop; production solver is native Rust.

## ADR-008 — LLM off the critical path, constrained, same validation
**Context.** A non-deterministic LLM must never be in the chain that produces certified math.
**Decision.** The form is the source of truth; the LLM is a convenience on-ramp that emits the
*same* JSON via **constrained/structured decoding** against the schema, from a **fixed enum library
of mechanics**, and is then validated by the *same* Pydantic/schema path. A hallucinated config can
at worst produce a *wrong-but-real* game — never unsafe code or corrupted math.
**Consequences.** LLM provider is non-foundational and deferred.

## ADR-009 — Export-boundary float discipline
**Context.** Even with integer accounting, σ / percentages / confidence intervals on the math sheet
use floating point, which differs across arch/compiler/FMA settings — but sheets are compared
digit-for-digit.
**Decision.** Integers/rationals everywhere possible; for unavoidable float, **fix the rounding
mode, render via a decimal library, pin the toolchain, never `-ffast-math`**. Compute variance via
integer sum-of-squares.
**Consequences.** Reproducible sheets across machines.

## ADR-010 — Reframed Math Parity Test
**Context.** The original global gate ("1B spins, <0.05% deviation") is statistically unreachable
for high-vol games (ADR-001).
**Decision.** Pass = **exact-math RTP matches the certified sheet** (target <0.001%); MC metrics are
reported **± confidence interval** and must *contain* the certified value. Full methodology in
[`parity-test.md`](parity-test.md). **Run at the end of M1**, not M4.

## ADR-011 — Declared, certified recursion caps for feature-buy / retriggers
**Context.** Retriggers and feature-buys are recursive and consume unbounded RNG draws; labs ask
"what is the theoretical maximum sequence length?".
**Decision.** Hard, **declared** depth/iteration caps that are themselves certified parameters.
Feature-buy RTP is reported as a **separate** number per entry mode.

## ADR-012 — Streaming integer statistics with fixed accumulator schema
**Context.** You cannot retain per-spin data across 10⁹–10¹² spins.
**Decision.** Per-thread integer accumulators: count, Σwin, Σwin² (`u128`), a **fixed log-scale
win-distribution histogram** (0–5000×), per-payline and per-symbol hit counters, feature-trigger
counters, and a max-win tracker — merged associatively. The histogram/counter schema is fixed up
front (it's a design decision, not an afterthought) because it drives volatility/percentile and the
GLI sheet.
