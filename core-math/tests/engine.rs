//! M1 acceptance tests:
//!  1. determinism — Monte Carlo is identical to the unit regardless of thread count (ADR-003)
//!  2. exact vs MC — the simulator agrees with the exact math within statistical noise (ADR-001)
//!  3. hand-verifiable uncapped — exact line ratio matches a by-hand computation (72/27)
//!  4. cap fold-in — exact capped enumeration matches a by-hand capped value (40/27) and MC (ADR-005)
//!  5. cross-check — the enumeration's uncapped sum equals the closed-form RTP
//!  6. weight-reels — weights realize to the equivalent strip and give identical exact math (ADR-006)

use sas_math_core::config::GameConfig;
use sas_math_core::enumerate::{enumerate_capped, exact_rtp};
use sas_math_core::mechanics::Prepared;
use sas_math_core::simulate::{monte_carlo, monte_carlo_threads};

const SAMPLE: &str = "examples/sample_5x3.json";

fn load(path: &str) -> GameConfig {
    let text = std::fs::read_to_string(path).expect("read config");
    GameConfig::from_json_str(&text).expect("parse config")
}

// 3-reel x 1-row, 1 payline; strips [A,A,B]; A x3 pays 9; bet 1. P(AAA)=8/27. cap as given.
fn tiny_strips(cap_x: f64) -> GameConfig {
    let json = format!(
        r#"{{
      "schema_version":"0.1.0",
      "game":{{"name":"tiny","grid":{{"reels":3,"rows":1}},"bet_units":1,"denomination_scale":1}},
      "symbols":[{{"id":0,"name":"A","kind":"regular"}},{{"id":1,"name":"B","kind":"regular"}}],
      "reels":{{"strips":[[0,0,1],[0,0,1],[0,0,1]]}},
      "paytable":[{{"symbol_id":0,"pays":{{"3":9}}}}],
      "paylines":[[0,0,0]],
      "line_eval":"left_to_right",
      "max_win":{{"cap_x":{cap_x}}},
      "rng":{{"generator":"chacha","master_seed":1}}
    }}"#
    );
    GameConfig::from_json_str(&json).unwrap()
}

#[test]
fn determinism_is_thread_independent() {
    let cfg = load(SAMPLE);
    let prep = Prepared::new(&cfg).unwrap();
    let seed = cfg.rng.master_seed;
    let n = 1_000_000u64;

    let a = monte_carlo_threads(&prep, seed, n, 1);
    let b = monte_carlo_threads(&prep, seed, n, 4);

    assert_eq!(a.total_won, b.total_won, "total_won differs across thread counts");
    assert_eq!(a.total_bet, b.total_bet);
    assert_eq!(a.sum_sq_win, b.sum_sq_win);
    assert_eq!(a.hits, b.hits);
    assert_eq!(a.max_win, b.max_win);
    assert_eq!(a.hist, b.hist, "win histogram differs across thread counts");

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
    let baseline = exact.capped.unwrap_or(exact.rtp); // sample cap is non-binding
    let diff = (mc - baseline).abs();

    assert!(
        diff <= 6.0 * sigma_mean + 1e-9,
        "exact {:.6} vs MC {:.6}: diff {:.6} exceeds 6*sigma_mean {:.6}",
        baseline,
        mc,
        diff,
        6.0 * sigma_mean
    );
}

#[test]
fn hand_verifiable_uncapped() {
    let cfg = tiny_strips(1_000_000.0);
    let prep = Prepared::new(&cfg).unwrap();
    let exact = exact_rtp(&prep);

    // exact line ratio, by hand: 9 * (2*2*2) / 3^3 = 72/27
    assert_eq!(exact.line_ratio.num, 72);
    assert_eq!(exact.line_ratio.den, 27);
    assert!((exact.rtp - 72.0 / 27.0).abs() < 1e-12);

    let n = 2_000_000u64;
    let acc = monte_carlo(&prep, 12345, n);
    let sigma_mean = (acc.return_variance(prep.bet_units) / n as f64).sqrt();
    assert!((acc.rtp() - exact.rtp).abs() <= 6.0 * sigma_mean + 1e-9);
}

#[test]
fn cap_fold_in_is_exact() {
    // cap_x = 5, bet = 1 -> each AAA atom (win 9) is capped to 5.
    // exact capped RTP = 5 * 8/27 = 40/27;  uncapped = 72/27.
    let cfg = tiny_strips(5.0);
    let prep = Prepared::new(&cfg).unwrap();
    let ce = enumerate_capped(&prep).expect("tiny game is enumerable");

    assert!((ce.uncapped_rtp - 72.0 / 27.0).abs() < 1e-12, "cross-check uncapped");
    assert!((ce.capped_rtp - 40.0 / 27.0).abs() < 1e-12, "exact capped value");
    assert!(ce.capped_rtp < ce.uncapped_rtp, "cap must erode RTP");

    // Monte Carlo (which applies the cap) must match the exact capped value.
    let n = 3_000_000u64;
    let acc = monte_carlo(&prep, 7, n);
    let sigma_mean = (acc.return_variance(prep.bet_units) / n as f64).sqrt();
    assert!((acc.rtp() - ce.capped_rtp).abs() <= 6.0 * sigma_mean + 1e-9);
}

#[test]
fn enumeration_uncapped_equals_closed_form() {
    let cfg = load(SAMPLE);
    let prep = Prepared::new(&cfg).unwrap();
    let exact = exact_rtp(&prep);
    let ce = enumerate_capped(&prep).expect("sample is enumerable");

    // the SAME enumeration, uncapped, must reproduce the closed-form marginal RTP exactly
    assert!(
        (ce.uncapped_rtp - exact.rtp).abs() < 1e-9,
        "enum uncapped {} != closed-form {}",
        ce.uncapped_rtp,
        exact.rtp
    );
    // sample cap is non-binding, so capped == uncapped
    assert!((ce.capped_rtp - exact.rtp).abs() < 1e-9);
}

#[test]
fn weights_realize_to_equivalent_strip() {
    // per-reel weights {0:2, 1:1} spread to strip [0,0,1] — identical to the tiny strips game.
    let json = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"tiny-weighted","grid":{"reels":3,"rows":1},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
      "reels":{"weights":[{"0":2,"1":1},{"0":2,"1":1},{"0":2,"1":1}]},
      "paytable":[{"symbol_id":0,"pays":{"3":9}}],
      "paylines":[[0,0,0]],
      "line_eval":"left_to_right",
      "max_win":{"cap_x":1000000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let cfg = GameConfig::from_json_str(json).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    assert!(prep.from_weights, "should be realized from weights");

    let exact = exact_rtp(&prep);
    assert_eq!(exact.line_ratio.num, 72, "weights must give the same exact math as strips");
    assert_eq!(exact.line_ratio.den, 27);

    let n = 2_000_000u64;
    let acc = monte_carlo(&prep, 99, n);
    let sigma_mean = (acc.return_variance(prep.bet_units) / n as f64).sqrt();
    assert!((acc.rtp() - exact.rtp).abs() <= 6.0 * sigma_mean + 1e-9);
}
