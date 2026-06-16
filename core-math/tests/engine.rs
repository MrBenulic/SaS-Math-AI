//! M1 acceptance tests:
//!  1. determinism — Monte Carlo is identical to the unit regardless of thread count (ADR-003)
//!  2. exact vs MC — the simulator agrees with the exact math within statistical noise (ADR-001)
//!  3. hand-verifiable — exact RTP matches a by-hand computation to the integer ratio

use sas_math_core::config::GameConfig;
use sas_math_core::enumerate::exact_rtp;
use sas_math_core::mechanics::Prepared;
use sas_math_core::simulate::{monte_carlo, monte_carlo_threads};

const SAMPLE: &str = "examples/sample_5x3.json";

fn load(path: &str) -> GameConfig {
    let text = std::fs::read_to_string(path).expect("read config");
    GameConfig::from_json_str(&text).expect("parse config")
}

#[test]
fn determinism_is_thread_independent() {
    let cfg = load(SAMPLE);
    let prep = Prepared::new(&cfg).unwrap();
    let seed = cfg.rng.master_seed;
    let n = 1_000_000u64;

    let a = monte_carlo_threads(&prep, seed, n, 1);
    let b = monte_carlo_threads(&prep, seed, n, 4);

    // Every integer accumulator must match exactly, to the unit.
    assert_eq!(a.total_won, b.total_won, "total_won differs across thread counts");
    assert_eq!(a.total_bet, b.total_bet);
    assert_eq!(a.sum_sq_win, b.sum_sq_win);
    assert_eq!(a.hits, b.hits);
    assert_eq!(a.max_win, b.max_win);
    assert_eq!(a.hist, b.hist, "win histogram differs across thread counts");

    // and a plain re-run is identical too
    let c = monte_carlo(&prep, seed, n);
    assert_eq!(a.total_won, c.total_won);
}

#[test]
fn exact_matches_monte_carlo() {
    let cfg = load(SAMPLE);
    let prep = Prepared::new(&cfg).unwrap();
    let exact = exact_rtp(&prep);

    let n = 4_000_000u64;
    let acc = monte_carlo(&prep, cfg.rng.master_seed, n);
    let mc = acc.rtp();
    let sigma_mean = (acc.return_variance(prep.bet_units) / n as f64).sqrt();
    let diff = (mc - exact.rtp).abs();

    assert!(
        diff <= 6.0 * sigma_mean + 1e-9,
        "exact {:.6} vs MC {:.6}: diff {:.6} exceeds 6*sigma_mean {:.6}",
        exact.rtp,
        mc,
        diff,
        6.0 * sigma_mean
    );
}

#[test]
fn hand_verifiable_exact_rtp() {
    // 3 reels x 1 row, 1 payline. Each strip = [A,A,B] (len 3). Pay A x3 = 9 units. bet = 1.
    // P(AAA) = (2/3)^3 = 8/27;  E[win] = 9 * 8/27 = 72/27;  RTP = 72/27 = 2.666...
    let json = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"tiny","grid":{"reels":3,"rows":1},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
      "reels":{"strips":[[0,0,1],[0,0,1],[0,0,1]]},
      "paytable":[{"symbol_id":0,"pays":{"3":9}}],
      "paylines":[[0,0,0]],
      "line_eval":"left_to_right",
      "max_win":{"cap_x":1000000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let cfg = GameConfig::from_json_str(json).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    let exact = exact_rtp(&prep);

    // exact integer ratio, computed by hand
    assert_eq!(exact.rtp_ratio.num, 72);
    assert_eq!(exact.rtp_ratio.den, 27);
    assert!((exact.rtp - 72.0 / 27.0).abs() < 1e-12);

    // Monte Carlo of the same tiny game agrees within noise
    let n = 2_000_000u64;
    let acc = monte_carlo(&prep, 12345, n);
    let sigma_mean = (acc.return_variance(prep.bet_units) / n as f64).sqrt();
    assert!((acc.rtp() - exact.rtp).abs() <= 6.0 * sigma_mean + 1e-9);
}
