# The Math Parity Test (reframed)

The global "is the engine solid?" gate. The blueprint's original version —
*"run 1B spins, require RTP/hit-freq/bonus-freq within 0.05% of a certified sheet"* — is
**statistically unreachable** for real games and is therefore replaced. See ADR-001 / ADR-010.

## Why the original gate fails
A Monte Carlo RTP estimate has standard error `SE ≈ σ / √N`. For a high-volatility game (max win
5000×), per-spin payout σ is easily 10–30× the mean, with a heavy tail. To get the RTP *confidence
interval* below 0.05% of RTP can need **10¹⁰–10¹² spins**, and **bonus-frequency / max-win-frequency
may not converge at 1B at all**. So "1B spins < 0.05%" tests luck, not correctness.

## The reframed test

**Primary gate — exact math parity (pass/fail):**
1. Take a known, already-certified live game.
2. Recreate its exact parameters (paytable, reel strips, paylines, features) as a `game-config.json`.
3. Compute **exact theoretical RTP** (and exact line/scatter hit-frequencies) by enumeration /
   convolution, with the max-win cap modeled *inside* the math (ADR-005).
4. **Pass** = exact RTP matches the certified sheet to **< 0.001%** (exact math should match to many
   decimals if the model is right). Per-payline and per-symbol exact hit-frequencies must match too.

**Secondary check — harness validation (Monte Carlo):**
5. Run deterministic MC on the *same* config. Every MC metric is reported with a **confidence
   interval**; the CI must **contain** both the exact value and the certified value. This proves the
   simulator and the exact engine agree, and that the RNG/accounting are sound — it does **not** try
   to beat exact math on precision.

**Reproducibility checks (must pass before parity is meaningful):**
6. **Seed replay:** run 10M spins with a fixed seed; record the final integer balance. Repeat 50×
   (and across different thread counts). Result must be **identical to the unit** every time.
7. **Stability under code change:** after adding a feature, base-game results for a given seed are
   unchanged (validates RNG namespacing, ADR-003).

## What routes to exact vs Monte Carlo
- **Exact (enumeration/convolution):** base-game line/scatter pays, finite free-spin models,
  anything with a tractable finite state space (e.g. 5×3 lines ≈ 32⁵).
- **Monte Carlo (+ Markov/absorbing-state where possible):** cascades/tumbles, hold-and-spin,
  symbol-collection metas, unbounded-retrigger features — the non-enumerable parts only.

## When the test runs
At the **end of M1** (not M4) — running it early de-risks the entire foundation (RNG namespacing,
integer accounting, line-eval semantics, enumeration) before the solver, AI, and features are built
on top of it.
