//! M2 solver acceptance tests:
//!  1. consistency — the solver's closed-form LINE rtp equals the engine's exact line rtp
//!  2. the gate — a broken ~140% game tunes to 96% and re-verifies against the integer engine
//!  3. non-degeneracy — the tuned game keeps a realistic hit-frequency
//!  4. determinism — same seed => identical tuned weights

use sas_math_core::config::GameConfig;
use sas_math_core::enumerate::exact_rtp;
use sas_math_core::mechanics::Prepared;
use sas_math_core::simulate::monte_carlo;
use sas_math_core::solver::{closed_form_rtp, realized_config, solve, SolveOptions};

fn load(path: &str) -> GameConfig {
    let text = std::fs::read_to_string(path).expect("read config");
    GameConfig::from_json_str(&text).expect("parse config")
}

#[test]
fn closed_form_line_matches_engine() {
    // The solver's closed-form line RTP (from the strip-count weights) must equal the engine's
    // exact line RTP — both are the same exact per-reel marginal.
    let cfg = load("examples/sample_5x3.json");
    let prep = Prepared::new(&cfg).unwrap();
    let exact = exact_rtp(&prep);
    let w: Vec<Vec<f64>> = (0..prep.reels)
        .map(|r| (0..prep.n_symbols).map(|s| prep.strip_counts[r][s] as f64).collect())
        .collect();
    let (_total, line, _scatter) = closed_form_rtp(&prep, &w);
    assert!(
        (line - exact.line_rtp).abs() < 1e-9,
        "closed-form line {} != engine line {}",
        line,
        exact.line_rtp
    );
}

#[test]
fn solver_hits_target_and_reverifies() {
    let cfg = load("examples/broken_lineonly.json");
    let prep = Prepared::new(&cfg).unwrap();
    let target = 0.96;
    let opts = SolveOptions { generations: 300, ..Default::default() };
    let res = solve(&prep, target, &opts);

    // closed-form must land near the target
    assert!(
        (res.total_rtp - target).abs() < 0.01,
        "closed-form {} not within 1% of {}",
        res.total_rtp,
        target
    );

    // realize tuned weights to integer strips and re-verify with the exact integer engine
    let tuned = realized_config(&cfg, &res.weights, 512.0);
    let tprep = Prepared::new(&tuned).unwrap();
    let texact = exact_rtp(&tprep);
    assert!(
        (texact.rtp - target).abs() < 0.02,
        "realized exact RTP {} not within 2% of {}",
        texact.rtp,
        target
    );
}

#[test]
fn tuned_game_is_not_degenerate() {
    let cfg = load("examples/broken_lineonly.json");
    let prep = Prepared::new(&cfg).unwrap();
    let res = solve(&prep, 0.96, &SolveOptions { generations: 300, ..Default::default() });
    let tuned = realized_config(&cfg, &res.weights, 512.0);
    let tprep = Prepared::new(&tuned).unwrap();
    let mc = monte_carlo(&tprep, 12345, 1_000_000);
    let hf = mc.hit_frequency();
    assert!(hf > 0.05 && hf < 0.95, "hit frequency {} is degenerate", hf);
}

#[test]
fn solver_is_deterministic() {
    let cfg = load("examples/broken_lineonly.json");
    let prep = Prepared::new(&cfg).unwrap();
    let opts = SolveOptions { generations: 100, ..Default::default() };
    let a = solve(&prep, 0.96, &opts);
    let b = solve(&prep, 0.96, &opts);
    assert_eq!(a.weights, b.weights, "same seed must give identical tuned weights");
    assert_eq!(a.total_rtp.to_bits(), b.total_rtp.to_bits());
}
