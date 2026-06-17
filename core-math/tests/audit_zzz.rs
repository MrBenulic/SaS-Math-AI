use sas_math_core::config::GameConfig;
use sas_math_core::mechanics::Prepared;
use sas_math_core::solver::{solve, closed_form_rtp, weights_to_integer, SolveOptions};

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

#[test]
fn ways_solver_is_blind() {
    let cfg = GameConfig::from_json_str(WAYS_WEIGHTS).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    // build init weights from sym_count
    let w: Vec<Vec<f64>> = (0..prep.reels)
        .map(|r| (0..prep.n_symbols).map(|s| prep.sym_count[r][s] as f64).collect())
        .collect();
    let cf = closed_form_rtp(&prep, &w);
    println!("CONFIRM closed_form RTP for ways game (true ways RTP is 3.625): {:?}", cf);
    let opts = SolveOptions { generations: 20, ..SolveOptions::default() };
    let res = solve(&prep, 0.96, &opts);
    println!("CONFIRM solver total_rtp reported: {}  feasibility: {:?}", res.total_rtp, res.feasibility);
    assert!(cf.0.abs() < 1e-9, "closed-form RTP for ways game is 0 -> solver blind");
}

#[test]
fn rounding_can_drop_a_symbol() {
    // A reel where one symbol has a tiny relative weight: scale 2048, ratio < 1/4096 rounds to 0,
    // but max(1.0) forces it to 1 -> floor inflation. Conversely a symbol meant to be present can be
    // duplicated/lost. Show weight not preserved exactly.
    let w = vec![vec![1.0, 4095.0]]; // ratios 1/4096 and 4095/4096
    let m = weights_to_integer(&w, 2048.0);
    println!("CONFIRM rounding: {:?}", m[0]);
    // symbol 0: round(2048/4096)=round(0.5)=0 -> max(1)=1 ; symbol1: round(2048*4095/4096)=round(2047.5)=2048
    // The realized ratio 1:2048 != intended 1:4095 -> weight distorted.
    let s0 = m[0].get("0").copied().unwrap_or(0);
    let s1 = m[0].get("1").copied().unwrap_or(0);
    println!("realized ratio s1/s0 = {}/{} = {}; intended 4095", s1, s0, s1 as f64/s0 as f64);
}

#[test]
fn ways_solve_multi_final_panic_check() {
    // solve_multi with vol target on a ways game: realized_config still ways; Prepared::new ok.
    // Just ensure no panic but document it tunes a blind objective.
    let cfg = GameConfig::from_json_str(WAYS_WEIGHTS).unwrap();
    let prep = Prepared::new(&cfg).unwrap();
    let w: Vec<Vec<f64>> = (0..prep.reels)
        .map(|r| (0..prep.n_symbols).map(|s| prep.sym_count[r][s] as f64).collect())
        .collect();
    let cf = closed_form_rtp(&prep, &w);
    assert!(cf.0 == 0.0);
}
