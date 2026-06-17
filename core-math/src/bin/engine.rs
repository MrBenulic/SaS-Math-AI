//! JSON bridge: lets the Python ai-layer drive the engine/solver over stdin/stdout without PyO3.
//!
//! Request  (stdin, one JSON object):
//!   {"command":"simulate","config":{...},"spins":2000000}
//!   {"command":"tune","config":{...},"target_rtp":0.96,"generations":200,
//!    "target_volatility":3.0,"target_hit_frequency":0.30}
//! Response (stdout, one JSON object): metrics for `simulate`, or before/after + tuned_config for
//! `tune`. On error: {"error":"..."} with exit code 1.

use sas_math_core::config::GameConfig;
use sas_math_core::enumerate::exact_rtp;
use sas_math_core::mechanics::Prepared;
use sas_math_core::outcome::simulate_one_traced;
use sas_math_core::simulate::monte_carlo;
use sas_math_core::solver::{realized_config, solve, solve_multi, Feasibility, SolveOptions, SolveTargets};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::time::Instant;

#[derive(Deserialize)]
#[serde(tag = "command", rename_all = "lowercase")]
enum Request {
    Simulate {
        config: GameConfig,
        #[serde(default)]
        spins: Option<u64>,
    },
    Tune {
        config: GameConfig,
        target_rtp: f64,
        #[serde(default)]
        generations: Option<usize>,
        #[serde(default)]
        target_volatility: Option<f64>,
        #[serde(default)]
        target_hit_frequency: Option<f64>,
    },
    /// Replay one exact spin as a deterministic outcome/event stream (the art↔math bridge).
    Replay {
        config: GameConfig,
        spin_index: u64,
        #[serde(default)]
        seed: Option<u64>,
    },
}

#[derive(Serialize)]
struct SimResponse {
    exact_rtp: f64,
    line_rtp: f64,
    scatter_rtp: f64,
    // Integer-exact line RTP as a rational `num/den` (ADR-002/009). u128 rendered as decimal strings
    // so no precision is lost crossing JSON. This is the certification-grade number; the floats above
    // are convenience renders. (Scatter is integer for Strips, binomial-f64 for Weights — see note.)
    exact_line_ratio_num: String,
    exact_line_ratio_den: String,
    capped_rtp: Option<f64>,
    capped_provenance: String,
    exact_combos: String,
    mc_rtp: f64,
    mc_ci95: f64,
    hit_frequency: f64,
    volatility: f64,
    max_win_x: f64,
    spins: u64,
    throughput_mps: f64,
}

#[derive(Serialize)]
struct Metrics {
    exact_rtp: f64,
    mc_rtp: f64,
    hit_frequency: f64,
    volatility: f64,
}

#[derive(Serialize)]
struct TuneResponse {
    before: Metrics,
    after: Metrics,
    tuned_config: GameConfig,
    target_rtp: f64,
    solver_total_rtp: f64,
    generations: usize,
    evaluations: usize,
    realize_delta: f64,
    /// Did the solver reach the requested target(s) within tolerance? (false => over-constrained.)
    feasible: bool,
    /// Which target(s) the solver could not meet (empty when feasible).
    missed_targets: Vec<String>,
}

fn fail(msg: String) -> ! {
    println!("{}", serde_json::json!({ "error": msg }));
    std::process::exit(1);
}

fn metrics(prep: &Prepared, seed: u64, spins: u64) -> Metrics {
    let exact = exact_rtp(prep);
    let acc = monte_carlo(prep, seed, spins);
    Metrics {
        exact_rtp: exact.rtp,
        mc_rtp: acc.rtp(),
        hit_frequency: acc.hit_frequency(),
        volatility: acc.return_variance(prep.bet_units).sqrt(),
    }
}

fn main() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        fail("failed to read stdin".into());
    }
    let req: Request = match serde_json::from_str(&input) {
        Ok(r) => r,
        Err(e) => fail(format!("invalid request JSON: {e}")),
    };

    match req {
        Request::Simulate { config, spins } => {
            let prep = match Prepared::new(&config) {
                Ok(p) => p,
                Err(e) => fail(format!("invalid config: {e}")),
            };
            let spins = spins.unwrap_or(2_000_000);
            let exact = exact_rtp(&prep);
            let t = Instant::now();
            let acc = monte_carlo(&prep, config.rng.master_seed, spins);
            let secs = t.elapsed().as_secs_f64();
            let resp = SimResponse {
                exact_rtp: exact.rtp,
                line_rtp: exact.line_rtp,
                scatter_rtp: exact.scatter_rtp,
                exact_line_ratio_num: exact.line_ratio.num.to_string(),
                exact_line_ratio_den: exact.line_ratio.den.to_string(),
                capped_rtp: exact.capped,
                capped_provenance: exact.capped_provenance().to_string(),
                exact_combos: exact.combos.to_string(),
                mc_rtp: acc.rtp(),
                mc_ci95: acc.rtp_ci95(prep.bet_units),
                hit_frequency: acc.hit_frequency(),
                volatility: acc.return_variance(prep.bet_units).sqrt(),
                max_win_x: acc.max_win as f64 / prep.bet_units as f64,
                spins,
                throughput_mps: if secs > 0.0 { spins as f64 / secs / 1e6 } else { 0.0 },
            };
            println!("{}", serde_json::to_string(&resp).unwrap());
        }
        Request::Tune { config, target_rtp, generations, target_volatility, target_hit_frequency } => {
            let prep = match Prepared::new(&config) {
                Ok(p) => p,
                Err(e) => fail(format!("invalid config: {e}")),
            };
            let seed = config.rng.master_seed;
            let before = metrics(&prep, seed, 1_000_000);

            let opts = SolveOptions { generations: generations.unwrap_or(200), ..SolveOptions::default() };
            let multi = target_volatility.is_some() || target_hit_frequency.is_some();
            let res = if multi {
                let targets = SolveTargets { rtp: target_rtp, volatility: target_volatility, hit_frequency: target_hit_frequency };
                solve_multi(&prep, &config, &targets, &opts)
            } else {
                solve(&prep, target_rtp, &opts)
            };

            let tuned_cfg = realized_config(&config, &res.weights, 2048.0);
            let tuned_prep = match Prepared::new(&tuned_cfg) {
                Ok(p) => p,
                Err(e) => fail(format!("invalid tuned config: {e}")),
            };
            let after = metrics(&tuned_prep, seed, 1_000_000);

            let (feasible, missed_targets) = match &res.feasibility {
                Feasibility::Feasible => (true, Vec::new()),
                Feasibility::Infeasible { missed, .. } => (false, missed.clone()),
            };

            let resp = TuneResponse {
                realize_delta: (after.exact_rtp - res.total_rtp).abs(),
                before,
                after,
                tuned_config: tuned_cfg,
                target_rtp,
                solver_total_rtp: res.total_rtp,
                generations: res.generations,
                evaluations: res.evaluations,
                feasible,
                missed_targets,
            };
            println!("{}", serde_json::to_string(&resp).unwrap());
        }
        Request::Replay { config, spin_index, seed } => {
            let prep = match Prepared::new(&config) {
                Ok(p) => p,
                Err(e) => fail(format!("invalid config: {e}")),
            };
            let seed = seed.unwrap_or(config.rng.master_seed);
            let outcome = simulate_one_traced(&prep, seed, spin_index);
            println!("{}", serde_json::to_string(&outcome).unwrap());
        }
    }
}
