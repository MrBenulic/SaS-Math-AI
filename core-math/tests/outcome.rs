//! Event-stream / replay acceptance tests (the art↔math bridge):
//!  1. `simulate_one == simulate_one_traced(..).total_win_units` over many spins — the trace can
//!     never silently diverge from the certified Monte Carlo aggregate (protects the ADR-010 gate).
//!  2. replay is deterministic and byte-stable (ADR-003).
//!  3. outcome internal invariants: parts sum to raw; cap applied correctly; cells in bounds.
//!  4. `both_ways` pays a full-line win once (not double); end-anchored wins still both pay.
//!  5. `ways_243` is rejected loudly rather than silently paying zero.

use sas_math_core::config::GameConfig;
use sas_math_core::mechanics::Prepared;
use sas_math_core::outcome::{simulate_one_traced, WinDirection};
use sas_math_core::simulate::simulate_one;

fn load(path: &str) -> GameConfig {
    let text = std::fs::read_to_string(path).expect("read config");
    GameConfig::from_json_str(&text).expect("parse config")
}

fn prep_both_ways_5x1() -> Prepared {
    // 5x1, both_ways. A(0) pays {2:10, 5:100}; B(1) never pays. Strips are irrelevant here because
    // the tests call `line_pay` directly on hand-built lines.
    let json = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"bw","grid":{"reels":5,"rows":1},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
      "reels":{"strips":[[0,1],[0,1],[0,1],[0,1],[0,1]]},
      "paytable":[{"symbol_id":0,"pays":{"2":10,"5":100}}],
      "paylines":[[0,0,0,0,0]],
      "line_eval":"both_ways",
      "max_win":{"cap_x":1000000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    Prepared::new(&GameConfig::from_json_str(json).unwrap()).unwrap()
}

#[test]
fn traced_total_matches_scalar() {
    // Covers the Strips model (sample_5x3) and a binding cap (sample_5x3_cap50).
    for path in ["examples/sample_5x3.json", "examples/sample_5x3_cap50.json"] {
        let cfg = load(path);
        let prep = Prepared::new(&cfg).unwrap();
        let seed = cfg.rng.master_seed;
        for spin in 0..200_000u64 {
            let scalar = simulate_one(&prep, seed, spin);
            let traced = simulate_one_traced(&prep, seed, spin);
            assert_eq!(
                scalar, traced.total_win_units,
                "scalar vs traced mismatch at spin {spin} in {path}"
            );
        }
    }
}

#[test]
fn replay_is_deterministic_and_stable() {
    let cfg = load("examples/sample_5x3.json");
    let prep = Prepared::new(&cfg).unwrap();
    let seed = cfg.rng.master_seed;
    for spin in [0u64, 1, 42, 9_999, 1_000_000] {
        let a = simulate_one_traced(&prep, seed, spin);
        let b = simulate_one_traced(&prep, seed, spin);
        assert_eq!(a, b, "replay diverged at spin {spin}");
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap(),
            "serialized replay not byte-stable at spin {spin}"
        );
    }
}

#[test]
fn outcome_invariants_hold() {
    let cfg = load("examples/sample_5x3_cap50.json");
    let prep = Prepared::new(&cfg).unwrap();
    let seed = cfg.rng.master_seed;
    for spin in 0..100_000u64 {
        let o = simulate_one_traced(&prep, seed, spin);

        // parts (all line wins + scatter) sum to the raw (pre-cap) total
        let parts: u64 = o.line_wins.iter().map(|w| w.pay_units).sum::<u64>()
            + o.scatter.as_ref().map(|s| s.pay_units).unwrap_or(0);
        assert_eq!(parts, o.raw_win_units, "parts != raw at spin {spin}");

        // cap modeling is consistent
        assert_eq!(o.total_win_units, o.raw_win_units.min(prep.cap_units));
        assert_eq!(o.cap_applied, o.total_win_units < o.raw_win_units);

        // cells are in bounds and consistent with the run length
        for w in &o.line_wins {
            assert_eq!(w.cells.len(), w.count, "cell count != run length at spin {spin}");
            for &(r, row) in &w.cells {
                assert!(r < prep.reels && row < prep.rows, "cell out of bounds at spin {spin}");
            }
        }
        assert_eq!(o.grid.len(), prep.reels);
    }
}

#[test]
fn both_ways_full_line_paid_once() {
    let prep = prep_both_ways_5x1();
    // Full line of A (5-of-a-kind) pays 100 ONCE — not 200 from summing both directions.
    assert_eq!(prep.line_pay(&[0, 0, 0, 0, 0]), 100);
    // A,A,B,A,A: a 2-of-a-kind from the left AND a 2-of-a-kind from the right are two distinct wins.
    assert_eq!(prep.line_pay(&[0, 0, 1, 0, 0]), 20);
    // Only a left-anchored 2-of-a-kind.
    assert_eq!(prep.line_pay(&[0, 0, 1, 1, 1]), 10);
    // Only a right-anchored 2-of-a-kind (caught by the reverse pass).
    assert_eq!(prep.line_pay(&[1, 1, 1, 0, 0]), 10);
}

#[test]
fn both_ways_traced_matches_line_pay() {
    let prep = prep_both_ways_5x1();
    // The traced line_wins pays must reconcile to line_pay for representative lines.
    for line in [
        [0usize, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 1, 1],
        [1, 1, 1, 0, 0],
    ] {
        let sum: u64 = prep.line_wins(&line).iter().map(|d| d.pay).sum();
        assert_eq!(sum, prep.line_pay(&line), "line_wins != line_pay for {line:?}");
    }
}

#[test]
fn golden_replay_sample_5x3_spin1() {
    // Pin an exact spin: any code change that perturbs a seed's outcome is caught here (ADR-003
    // stability-under-code-change). Values are produced by the engine itself and are deterministic
    // cross-platform (integer accounting + ChaCha8 counter-RNG).
    let cfg = load("examples/sample_5x3.json");
    let prep = Prepared::new(&cfg).unwrap();
    let o = simulate_one_traced(&prep, cfg.rng.master_seed, 1);

    assert_eq!(o.total_win_units, 24);
    assert_eq!(
        o.grid,
        vec![vec![4, 5, 6], vec![5, 6, 3], vec![4, 5, 6], vec![5, 6, 0], vec![3, 4, 5]]
    );
    assert_eq!(o.line_wins.len(), 1);
    let w = &o.line_wins[0];
    assert_eq!((w.line_index, w.symbol, w.count, w.pay_units), (5, 5, 4, 24));
    assert_eq!(w.direction, WinDirection::LeftToRight);
    assert!(o.scatter.is_none());
    assert!(!o.cap_applied);
}

#[test]
fn ways_243_is_rejected_loudly() {
    let json = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"w","grid":{"reels":3,"rows":3},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"}],
      "reels":{"strips":[[0],[0],[0]]},
      "paytable":[{"symbol_id":0,"pays":{"3":9}}],
      "paylines":[[0,1,2]],
      "line_eval":"ways_243",
      "max_win":{"cap_x":1000000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let result = Prepared::new(&GameConfig::from_json_str(json).unwrap());
    assert!(result.is_err(), "ways_243 must be rejected, not silently zero-paying");
}
