# core-math (Rust) — Module 3 + 4

The deterministic math engine. **Not yet implemented** — built in **M1** (`v0.2.0`).
This is a **pure Rust crate** (no Python/FFI in the core; ADR-007).

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
