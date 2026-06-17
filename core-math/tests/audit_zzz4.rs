use sas_math_core::config::GameConfig;
use sas_math_core::mechanics::Prepared;
use sas_math_core::solver::{solve_multi, SolveOptions, SolveTargets, weights_to_integer, realized_config};

// Can a reel end up all-zero so weights_to_integer yields an empty map -> Prepared::new Err ->
// solve_multi's .expect("prepare final") PANIC? 
// weights_from_x sets only `active` (initial>0) cells via exp() (>0). A reel with at least one active
// cell stays positive. But realized_config also re-prepares INSIDE the loop with .ok() (safe). The
// FINAL one uses .expect. Construct weights with a fully-zero reel directly:
#[test]
fn weights_to_integer_empty_reel_then_prepare_panics() {
    let base = GameConfig::from_json_str(SAMPLE).unwrap();
    // weights vec: reel 1 is all zeros
    let mut w = vec![vec![1.0, 1.0], vec![0.0, 0.0]];
    let m = weights_to_integer(&w, 100.0);
    println!("reel0 map={:?} reel1 map={:?}", m[0], m[1]);
    assert!(m[1].is_empty(), "all-zero reel -> empty weight map");
    let cfg2 = realized_config(&base, &w, 100.0);
    let r = Prepared::new(&cfg2);
    println!("Prepared::new on config with empty reel weight map: {:?}", r.as_ref().err());
    assert!(r.is_err(), "empty reel rejected -> .expect() in solve_multi would PANIC");
    let _ = &mut w;
}

const SAMPLE: &str = r#"{
  "schema_version":"0.1.0",
  "game":{"name":"x","grid":{"reels":2,"rows":1},"bet_units":1},
  "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
  "reels":{"weights":[{"0":1,"1":1},{"0":1,"1":1}]},
  "paytable":[{"symbol_id":0,"pays":{"2":5}}],
  "paylines":[[0,0]],
  "line_eval":"left_to_right",
  "max_win":{"cap_x":1000},
  "rng":{"generator":"chacha","master_seed":1}
}"#;

// NaN propagation: if a target_rtp is 0, solve_multi divides by targets.rtp -> ((rtp-0)/0)^2 = inf/nan.
#[test]
fn zero_target_rtp_div() {
    let base = GameConfig::from_json_str(SAMPLE).unwrap();
    let prep = Prepared::new(&base).unwrap();
    let opts = SolveOptions { generations: 5, ..SolveOptions::default() };
    let targets = SolveTargets { rtp: 0.0, volatility: None, hit_frequency: None };
    let res = solve_multi(&prep, &base, &targets, &opts);
    println!("target_rtp=0 -> loss uses /0. final_loss={}, total_rtp={}, feas={:?}", res.final_loss, res.total_rtp, res.feasibility);
}
