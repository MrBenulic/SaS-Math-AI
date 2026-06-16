//! Auto-tuner CLI: load a config + target RTP, tune symbol weights, re-verify with the exact
//! integer engine + Monte Carlo, and write the tuned config.
//!
//! Usage: tune <config.json> [target_rtp=0.96] [generations=250]

use sas_math_core::config::GameConfig;
use sas_math_core::enumerate::exact_rtp;
use sas_math_core::mechanics::Prepared;
use sas_math_core::simulate::monte_carlo;
use sas_math_core::solver::{realized_config, solve, SolveOptions};
use std::time::Instant;

const VERIFY_SPINS: u64 = 2_000_000;
const SCALE: f64 = 512.0; // realized strip length per reel (higher = less rounding delta)

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.get(1) {
        Some(p) => p.clone(),
        None => {
            eprintln!("usage: tune <config.json> [target_rtp=0.96] [generations=250]");
            std::process::exit(2);
        }
    };
    let target: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.96);
    let gens: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(250);

    let text = std::fs::read_to_string(&path).expect("read config");
    let cfg = GameConfig::from_json_str(&text).expect("parse config");
    let prep = Prepared::new(&cfg).expect("prepare");

    // before
    let before_mc = monte_carlo(&prep, cfg.rng.master_seed, VERIFY_SPINS);
    let before_exact = exact_rtp(&prep);

    // tune
    let opts = SolveOptions { generations: gens, ..SolveOptions::default() };
    let t = Instant::now();
    let res = solve(&prep, target, &opts);
    let solve_secs = t.elapsed().as_secs_f64();

    // realize + re-verify with the integer engine
    let tuned_cfg = realized_config(&cfg, &res.weights, SCALE);
    let tuned_prep = Prepared::new(&tuned_cfg).expect("prepare tuned");
    let after_exact = exact_rtp(&tuned_prep);
    let after_mc = monte_carlo(&tuned_prep, cfg.rng.master_seed, VERIFY_SPINS);

    println!("============================================================");
    println!(" SaS-Math AI — Auto-Tuner (M2)   |   {}", cfg.game.name);
    println!("============================================================");
    println!(" Target RTP:            {:>8.4}%", target * 100.0);
    println!("------------------------------------------------------------");
    println!(" BEFORE");
    println!("   exact RTP (uncapped): {:>8.4}%", before_exact.rtp * 100.0);
    println!("   MC RTP:               {:>8.4}%", before_mc.rtp() * 100.0);
    println!("   hit frequency:        {:>8.4}%", before_mc.hit_frequency() * 100.0);
    println!("------------------------------------------------------------");
    println!(" SOLVER  ({} generations, {} evals, {:.3}s)", res.generations, res.evaluations, solve_secs);
    println!("   closed-form tuned RTP:{:>8.4}%   (line {:.3}% + scatter {:.3}%)",
        res.total_rtp * 100.0, res.line_rtp * 100.0, res.scatter_rtp * 100.0);
    println!("------------------------------------------------------------");
    println!(" AFTER  (tuned weights realized to {}-stop strips, re-verified)", SCALE as u64);
    println!("   exact RTP (uncapped): {:>8.4}%   [closed-form]", after_exact.rtp * 100.0);
    println!("   MC RTP:               {:>8.4}%  +/- {:.4}%",
        after_mc.rtp() * 100.0, after_mc.rtp_ci95(tuned_prep.bet_units) * 100.0);
    println!("   hit frequency:        {:>8.4}%", after_mc.hit_frequency() * 100.0);
    println!("   |realized - target|:  {:>8.4}%", (after_exact.rtp - target).abs() * 100.0);
    println!("   realize delta (cf->engine): {:>7.4}%", (after_exact.rtp - res.total_rtp).abs() * 100.0);
    println!("------------------------------------------------------------");
    let playable = after_mc.hit_frequency() > 0.05 && after_mc.hit_frequency() < 0.95;
    println!(" Result: RTP {} target; hit-frequency {} (not degenerate)",
        if (after_exact.rtp - target).abs() < 0.005 { "HIT" } else { "near" },
        if playable { "realistic" } else { "CHECK" });

    let out = format!("{}.tuned.json", path);
    std::fs::write(&out, serde_json::to_string_pretty(&tuned_cfg).unwrap()).expect("write tuned");
    println!(" Tuned config written to: {}", out);
    println!("============================================================");
}
