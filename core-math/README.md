# core-math (Rust) — Module 3 + 4

The deterministic math engine **+ M2 auto-tuner**. On `beta`, tagged `v0.2.0-beta.2` (M1 engine) /
`v0.3.0-beta.2` (M2 solver). This is a **pure Rust crate** (no Python/FFI in the core; ADR-007).

**M1 status (verified, 9 tests green):** closed-form uncapped RTP **and exact capped enumeration**
(`min(win,cap)` per outcome atom; feasibility-bounded with a numeric-MC fallback labeled by
provenance; built-in cross-check that enumeration-uncapped == closed-form); **weight-reels** realized
to deterministic strips (ADR-006); exact == Monte Carlo within 95% CI; thread-independent determinism;
hand-verifiable ratios (72/27 uncapped, 40/27 capped); ~19M spins/s. Run it:
`cargo run --release --bin cli -- examples/sample_5x3_cap50.json 5000000` (shows cap-erosion).
**Remaining before stable `v0.2.0`:** parity against an external **certified** reference game
(`ways_243` line-eval is a minor stub).

**M2 auto-tuner (`solver.rs`, `bin/tune.rs`):** tunes per-reel symbol weights to **RTP + volatility +
hit-frequency** (RTP via the exact closed-form objective; vol/hit via CRN Monte Carlo, ADR-006), then
realizes integer **weighted reels** and re-verifies with the engine. Weighted reels use the
independent-cell model, so the solver and re-verify agree (scatter games re-verify with <1% delta).
Gate PASS — broken 139% → 96%:
`cargo run --release --bin tune -- examples/broken_143.json 0.96 250` (scatter), or
`cargo run --release --bin tune -- examples/broken_lineonly.json 0.96 150 3.0` (RTP + volatility 3.0).
Next: full CMA-ES covariance adaptation; PyO3 bindings.

## Planned layout
```
core-math/
├─ Cargo.toml
├─ src/
│  ├─ lib.rs          public API: load config -> {ExactResult, McResult}
│  ├─ config.rs       serde structs generated/validated from schema/game-config.schema.json (ADR-004)
│  ├─ rng.rs          counter-based, namespaced PRNG: draw(subsystem, spin_idx, draw_idx) (ADR-003)
│  ├─ accounting.rs   integer/u128 money + deterministic associative merge (ADR-002)
│  ├─ enumerate.rs    exact RTP via enumeration/convolution, cap modeled inside (ADR-001, ADR-005)
│  ├─ simulate.rs     deterministic Monte Carlo over Rayon
│  ├─ stats.rs        streaming integer accumulators + fixed histogram/counters (ADR-012)
│  ├─ mechanics/      the "Lego-block" library (line-eval LTR/ways/both-ways, scatter/wild rules)
│  └─ bin/cli.rs      CLI: run exact + MC on a config, emit numeric math sheet
└─ tests/
   ├─ determinism.rs  seed-replay: 50 runs, identical to the unit, across thread counts
   └─ parity.rs       Math Parity Test vs a known certified game (docs/parity-test.md)
```

## M1 acceptance
Exact RTP matches a certified sheet < 0.001%; seed-replay identical to the unit; 10M-spin MC < 2s;
parity test green. **Correctness of the RNG/accounting foundation gates performance work.**
