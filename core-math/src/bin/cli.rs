//! CLI: load a JSON game config, compute exact RTP + run Monte Carlo, print a numeric math sheet.
//!
//! Usage: cli <config.json> [spins]   (default spins = 10_000_000)

use sas_math_core::config::GameConfig;
use sas_math_core::enumerate::exact_rtp;
use sas_math_core::mechanics::Prepared;
use sas_math_core::simulate::monte_carlo;
use sas_math_core::stats::bucket_label;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.get(1) {
        Some(p) => p.clone(),
        None => {
            eprintln!("usage: cli <config.json> [spins]");
            std::process::exit(2);
        }
    };
    let spins: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10_000_000);

    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error reading {path}: {e}");
        std::process::exit(1);
    });
    let cfg = GameConfig::from_json_str(&text).unwrap_or_else(|e| {
        eprintln!("error parsing config: {e}");
        std::process::exit(1);
    });
    let prep = Prepared::new(&cfg).unwrap_or_else(|e| {
        eprintln!("invalid config: {e}");
        std::process::exit(1);
    });

    let exact = exact_rtp(&prep);

    let t = Instant::now();
    let acc = monte_carlo(&prep, cfg.rng.master_seed, spins);
    let secs = t.elapsed().as_secs_f64();

    let mc_rtp = acc.rtp();
    let ci = acc.rtp_ci95(prep.bet_units);
    let vol = acc.return_variance(prep.bet_units).sqrt();
    // Monte Carlo is capped; compare it to the exact CAPPED baseline when we have it.
    let exact_baseline = exact.capped.unwrap_or(exact.rtp);
    let within = (mc_rtp - exact_baseline).abs() <= ci.max(1e-9);
    let throughput = if secs > 0.0 { spins as f64 / secs / 1e6 } else { 0.0 };

    println!("============================================================");
    println!(" SaS-Math AI — Math Sheet (M1)   |   {}", cfg.game.name);
    println!("============================================================");
    println!(
        " Grid: {}x{}   Paylines: {}   Bet: {} units   Cap: {}x",
        cfg.game.grid.reels,
        cfg.game.grid.rows,
        cfg.paylines.len(),
        cfg.game.bet_units,
        cfg.max_win.cap_x
    );
    println!("------------------------------------------------------------");
    println!(" EXACT RTP (uncapped):  {:>8.4}%   [closed-form]", exact.rtp * 100.0);
    println!("   - line pays:         {:>8.4}%", exact.line_rtp * 100.0);
    println!("   - scatter pays:      {:>8.4}%", exact.scatter_rtp * 100.0);
    match exact.capped {
        Some(c) => {
            println!(
                " CAPPED RTP (cap {}x):  {:>8.4}%   [{}, {} combos]",
                cfg.max_win.cap_x,
                c * 100.0,
                exact.capped_provenance(),
                exact.combos
            );
            println!("   - cap erosion:       {:>8.4}%", (exact.rtp - c) * 100.0);
        }
        None => println!(
            " CAPPED RTP (cap {}x):  numeric tail (enumeration > {} combos) — see MC below [{}]",
            cfg.max_win.cap_x,
            sas_math_core::enumerate::ENUM_LIMIT,
            exact.capped_provenance()
        ),
    }
    println!("------------------------------------------------------------");
    println!(" MONTE CARLO  (N = {} spins)", spins);
    println!(
        " RTP:                   {:>8.4}%  +/- {:.4}%  (95% CI)",
        mc_rtp * 100.0,
        ci * 100.0
    );
    println!(" Hit frequency:         {:>8.4}%", acc.hit_frequency() * 100.0);
    println!(" Volatility (sigma):    {:>8.4}", vol);
    println!(
        " Max win seen:          {:>8.2}x",
        acc.max_win as f64 / prep.bet_units as f64
    );
    println!(" Throughput:            {:>8.2} M spins/s  ({:.3}s)", throughput, secs);
    println!(
        " Exact vs MC:           {}",
        if within {
            "WITHIN 95% CI  [OK]"
        } else {
            "outside CI (check seed/variance)"
        }
    );
    println!("------------------------------------------------------------");
    println!(" Win distribution (share of spins):");
    for i in 0..acc.hist.len() {
        let c = acc.hist[i];
        if c == 0 {
            continue;
        }
        println!(
            "   {:<14} {:>9.4}%",
            bucket_label(i),
            c as f64 / acc.spins as f64 * 100.0
        );
    }
    println!("============================================================");
    println!(" {}", exact.note);
}
