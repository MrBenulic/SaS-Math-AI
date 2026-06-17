//! 243-ways (all-ways) acceptance tests:
//!  1. exact ways RTP matches a BY-HAND value (direct enumeration over all grids), for both reel
//!     models — this is the parity-critical check (line_eval is the #1 place math goes wrong).
//!  2. exact ways RTP matches deterministic Monte Carlo within a confidence interval.
//!
//! Hand value: 3 reels x 1 row, symbols A/B equally likely, A pays {1:2, 2:5, 3:20}. Over the 8
//! equally-likely grids the run length (leading reels showing A) pays: run1 (A,B,*) x2 grids -> 2
//! each; run2 (A,A,B) x1 -> 5; run3 (A,A,A) x1 -> 20. Total 4+5+20 = 29 over 8 grids = 29/8 = 3.625.

use sas_math_core::config::GameConfig;
use sas_math_core::enumerate::exact_rtp;
use sas_math_core::mechanics::Prepared;
use sas_math_core::simulate::monte_carlo;

fn prep_from(json: &str) -> Prepared {
    Prepared::new(&GameConfig::from_json_str(json).unwrap()).unwrap()
}

const WAYS_WEIGHTS: &str = r#"{
  "schema_version":"0.1.0",
  "game":{"name":"ways-w","grid":{"reels":3,"rows":1},"bet_units":1,"denomination_scale":1},
  "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
  "reels":{"weights":[{"0":1,"1":1},{"0":1,"1":1},{"0":1,"1":1}]},
  "paytable":[{"symbol_id":0,"pays":{"1":2,"2":5,"3":20}}],
  "paylines":[[0,0,0]],
  "line_eval":"ways_243",
  "max_win":{"cap_x":1000000},
  "rng":{"generator":"chacha","master_seed":1}
}"#;

const WAYS_STRIPS: &str = r#"{
  "schema_version":"0.1.0",
  "game":{"name":"ways-s","grid":{"reels":3,"rows":1},"bet_units":1,"denomination_scale":1},
  "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
  "reels":{"strips":[[0,1],[0,1],[0,1]]},
  "paytable":[{"symbol_id":0,"pays":{"1":2,"2":5,"3":20}}],
  "paylines":[[0,0,0]],
  "line_eval":"ways_243",
  "max_win":{"cap_x":1000000},
  "rng":{"generator":"chacha","master_seed":1}
}"#;

const HAND_RTP: f64 = 29.0 / 8.0; // 3.625

#[test]
fn ways_exact_matches_hand_value() {
    for json in [WAYS_WEIGHTS, WAYS_STRIPS] {
        let prep = prep_from(json);
        assert!(prep.is_ways(), "config should use 243-ways evaluation");
        let e = exact_rtp(&prep);
        assert!(
            (e.rtp - HAND_RTP).abs() < 1e-9,
            "ways exact RTP {} != hand value {}",
            e.rtp,
            HAND_RTP
        );
    }
}

#[test]
fn ways_exact_matches_monte_carlo() {
    for json in [WAYS_WEIGHTS, WAYS_STRIPS] {
        let prep = prep_from(json);
        let e = exact_rtp(&prep);
        let n = 3_000_000u64;
        let acc = monte_carlo(&prep, 7, n);
        let sigma_mean = (acc.return_variance(prep.bet_units) / n as f64).sqrt();
        assert!(
            (acc.rtp() - e.rtp).abs() <= 6.0 * sigma_mean + 1e-9,
            "ways exact {} vs MC {} (6*sigma_mean {})",
            e.rtp,
            acc.rtp(),
            6.0 * sigma_mean
        );
    }
}

#[test]
fn ways_mc_is_thread_independent() {
    use sas_math_core::simulate::monte_carlo_threads;
    let prep = prep_from(WAYS_STRIPS);
    let a = monte_carlo_threads(&prep, 7, 500_000, 1);
    let b = monte_carlo_threads(&prep, 7, 500_000, 4);
    assert_eq!(a.total_won, b.total_won, "ways MC must be thread-independent (ADR-003)");
}
