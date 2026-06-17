//! Hardening — regression tests for audit-confirmed bugs + property/fuzz invariants over many
//! randomly-generated configs. Randomness is a seeded xorshift so failures are reproducible.

use std::collections::HashMap;

use sas_math_core::config::{
    Game, GameConfig, Grid, LineEval, MaxWin, PaytableEntry, Reels, RngConfig, Symbol, SymbolKind,
};
use sas_math_core::enumerate::{enumerate_capped, exact_rtp};
use sas_math_core::mechanics::Prepared;
use sas_math_core::outcome::simulate_one_traced;
use sas_math_core::simulate::{monte_carlo, monte_carlo_threads, simulate_one};
use sas_math_core::solver::closed_form_rtp;

// ── seeded PRNG (reproducible) ──
fn xs(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}
fn rin(state: &mut u64, lo: u64, hi: u64) -> u64 {
    lo + xs(state) % (hi - lo + 1)
}

fn pays_map(state: &mut u64, from: usize, to: usize) -> HashMap<String, u64> {
    let mut m = HashMap::new();
    for c in from..=to {
        m.insert(c.to_string(), rin(state, 1, 50));
    }
    m
}

/// Build a random but ALWAYS-VALID config: WILD(0)+SCATTER(1)+two regulars, strips or weights.
fn random_config(state: &mut u64) -> GameConfig {
    let reels = rin(state, 2, 4) as usize;
    let rows = rin(state, 1, 3) as usize;
    let bet = rin(state, 1, 10);
    let n = 4usize; // ids 0..3

    let symbols = vec![
        Symbol { id: 0, name: "WILD".into(), kind: SymbolKind::Wild, substitutes_for: vec![2, 3] },
        Symbol { id: 1, name: "SCATTER".into(), kind: SymbolKind::Scatter, substitutes_for: vec![] },
        Symbol { id: 2, name: "R2".into(), kind: SymbolKind::Regular, substitutes_for: vec![] },
        Symbol { id: 3, name: "R3".into(), kind: SymbolKind::Regular, substitutes_for: vec![] },
    ];

    let max_count = reels * rows;
    let paytable = vec![
        PaytableEntry { symbol_id: 0, pays: pays_map(state, 2, reels) },
        PaytableEntry { symbol_id: 2, pays: pays_map(state, 2, reels) },
        PaytableEntry { symbol_id: 3, pays: pays_map(state, 2, reels) },
        PaytableEntry { symbol_id: 1, pays: pays_map(state, 2, max_count.min(5).max(2)) }, // scatter
    ];

    let reels_model = if xs(state) % 2 == 0 {
        let strips: Vec<Vec<usize>> = (0..reels)
            .map(|_| {
                let len = rin(state, 4, 7) as usize;
                (0..len).map(|_| rin(state, 0, (n - 1) as u64) as usize).collect()
            })
            .collect();
        Reels { strips: Some(strips), weights: None }
    } else {
        let weights: Vec<HashMap<String, u64>> = (0..reels)
            .map(|_| {
                let mut m = HashMap::new();
                for id in 0..n {
                    m.insert(id.to_string(), rin(state, 1, 9));
                }
                m
            })
            .collect();
        Reels { strips: None, weights: Some(weights) }
    };

    let line_eval = match rin(state, 0, 2) {
        0 => LineEval::LeftToRight,
        1 => LineEval::BothWays,
        _ => LineEval::Ways243,
    };

    let nlines = rin(state, 1, 3) as usize;
    let paylines: Vec<Vec<usize>> = (0..nlines)
        .map(|_| (0..reels).map(|_| rin(state, 0, (rows - 1) as u64) as usize).collect())
        .collect();

    let cap_x = if xs(state) % 2 == 0 { rin(state, 3, 60) as f64 } else { 1_000_000.0 };

    GameConfig {
        schema_version: "0.1.0".into(),
        game: Game { name: "fuzz".into(), grid: Grid { reels, rows }, bet_units: bet, denomination_scale: 1 },
        symbols,
        reels: reels_model,
        paytable,
        paylines,
        line_eval,
        max_win: MaxWin { cap_x },
        rng: RngConfig { generator: "chacha".into(), master_seed: rin(state, 1, 1_000_000) },
        features: vec![],
        targets: None,
    }
}

// ───────────────────────────── property / fuzz ─────────────────────────────

#[test]
fn fuzz_engine_invariants_hold() {
    let mut state = 0x5A5_u64.wrapping_mul(0x9E3779B97F4A7C15) | 1;
    for case in 0..40u64 {
        let cfg = random_config(&mut state);
        let prep = Prepared::new(&cfg).unwrap_or_else(|e| panic!("case {case}: valid config rejected: {e}"));

        let exact = exact_rtp(&prep);
        assert!(exact.rtp.is_finite() && exact.rtp >= 0.0, "case {case}: exact rtp {} bad", exact.rtp);

        // capped <= uncapped (strips enumerable path)
        if let Some(ce) = enumerate_capped(&prep) {
            assert!(
                ce.capped_rtp <= ce.uncapped_rtp + 1e-9,
                "case {case}: capped {} > uncapped {}",
                ce.capped_rtp,
                ce.uncapped_rtp
            );
        }

        // scalar == traced; cap never exceeded; cells in bounds
        let seed = cfg.rng.master_seed;
        for spin in 0..2000u64 {
            let scalar = simulate_one(&prep, seed, spin);
            let o = simulate_one_traced(&prep, seed, spin);
            assert_eq!(scalar, o.total_win_units, "case {case} spin {spin}: scalar != traced");
            assert!(o.total_win_units <= prep.cap_units, "case {case} spin {spin}: cap exceeded");
            let parts: u64 = o.line_wins.iter().map(|w| w.pay_units).sum::<u64>()
                + o.scatter.as_ref().map(|s| s.pay_units).unwrap_or(0);
            assert_eq!(parts, o.raw_win_units, "case {case} spin {spin}: parts != raw");
        }

        // MC: no panic, finite, deterministic & thread-independent
        let n = 120_000u64;
        let a = monte_carlo(&prep, seed, n);
        assert!(a.rtp().is_finite(), "case {case}: MC rtp not finite");
        let b = monte_carlo_threads(&prep, seed, n, 4);
        assert_eq!(a.total_won, b.total_won, "case {case}: MC not thread-independent");

        // exact ≈ MC — only when comparable: either capped-exact exists (Strips enumerable, both apply
        // the cap), or the cap never bound during MC (so capped-MC ≈ uncapped-exact). For weights/ways
        // with a binding cap, capped-exact is intentionally a numeric tail, so the comparison is skipped.
        let comparable = exact.capped.is_some() || a.max_win < prep.cap_units;
        if comparable {
            let baseline = exact.capped.unwrap_or(exact.rtp);
            let sigma_mean = (a.return_variance(prep.bet_units) / n as f64).sqrt();
            assert!(
                (a.rtp() - baseline).abs() <= 8.0 * sigma_mean + 1e-6,
                "case {case}: exact {} vs MC {} (8σ̄ {})",
                baseline,
                a.rtp(),
                8.0 * sigma_mean
            );
        }
    }
}

// ───────────────────────────── regression tests ─────────────────────────────

fn cfg_json(json: &str) -> Result<Prepared, String> {
    Prepared::new(&GameConfig::from_json_str(json).expect("parse"))
}

const SCATTER_CFG: &str = r#"{
  "schema_version":"0.1.0",
  "game":{"name":"s","grid":{"reels":3,"rows":2},"bet_units":1,"denomination_scale":1},
  "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"S","kind":"scatter"}],
  "reels":{"strips":[[0,1],[0,1],[0,1]]},
  "paytable":[{"symbol_id":0,"pays":{"3":10}},{"symbol_id":1,"pays":{"2":5}}],
  "paylines":[[0,0,0]],
  "line_eval":"left_to_right","max_win":{"cap_x":1000},
  "rng":{"generator":"chacha","master_seed":1}
}"#;

fn with(json: &str, find: &str, repl: &str) -> String {
    json.replace(find, repl)
}

#[test]
fn rejects_negative_or_zero_cap() {
    assert!(cfg_json(&with(SCATTER_CFG, "\"cap_x\":1000", "\"cap_x\":-5")).is_err());
    assert!(cfg_json(&with(SCATTER_CFG, "\"cap_x\":1000", "\"cap_x\":0")).is_err());
}

#[test]
fn rejects_zero_bet_units() {
    assert!(cfg_json(&with(SCATTER_CFG, "\"bet_units\":1", "\"bet_units\":0")).is_err());
}

#[test]
fn rejects_zero_count_paytable_key() {
    assert!(cfg_json(&with(SCATTER_CFG, "\"pays\":{\"2\":5}", "\"pays\":{\"0\":5,\"2\":5}")).is_err());
}

#[test]
fn scatter_pay_zero_count_is_zero() {
    // even if a (bypassed) config had a 0-pay, the sink never pays on zero scatters
    let prep = cfg_json(SCATTER_CFG).unwrap();
    assert_eq!(prep.scatter_pay(0), 0);
    assert_eq!(prep.scatter_pay(2), 5);
}

#[test]
fn rejects_multiple_scatters() {
    let json = with(
        SCATTER_CFG,
        "{\"id\":1,\"name\":\"S\",\"kind\":\"scatter\"}",
        "{\"id\":1,\"name\":\"S\",\"kind\":\"scatter\"},{\"id\":2,\"name\":\"S2\",\"kind\":\"scatter\"}",
    );
    assert!(cfg_json(&json).is_err(), "two scatters must be rejected");
}

#[test]
fn rejects_empty_paylines_for_line_game() {
    assert!(cfg_json(&with(SCATTER_CFG, "\"paylines\":[[0,0,0]]", "\"paylines\":[]")).is_err());
}

#[test]
fn rejects_reel_weight_sum_over_u32() {
    let json = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"big","grid":{"reels":2,"rows":1},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
      "reels":{"weights":[{"0":5000000000,"1":1},{"0":1,"1":1}]},
      "paytable":[{"symbol_id":0,"pays":{"2":10}}],
      "paylines":[[0,0]],"line_eval":"left_to_right","max_win":{"cap_x":1000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    assert!(cfg_json(json).is_err(), "reel weight-sum over u32::MAX must be rejected");
}

#[test]
fn solver_ways_objective_is_nonzero() {
    // The closed-form objective must SEE ways pays (regression for the constant-zero tuning bug).
    let json = r#"{
      "schema_version":"0.1.0",
      "game":{"name":"w","grid":{"reels":3,"rows":2},"bet_units":1,"denomination_scale":1},
      "symbols":[{"id":0,"name":"A","kind":"regular"},{"id":1,"name":"B","kind":"regular"}],
      "reels":{"weights":[{"0":1,"1":1},{"0":1,"1":1},{"0":1,"1":1}]},
      "paytable":[{"symbol_id":0,"pays":{"2":5,"3":20}}],
      "paylines":[[0,0,0]],"line_eval":"ways_243","max_win":{"cap_x":1000000},
      "rng":{"generator":"chacha","master_seed":1}
    }"#;
    let prep = cfg_json(json).unwrap();
    let w: Vec<Vec<f64>> = (0..prep.reels)
        .map(|r| (0..prep.n_symbols).map(|s| prep.sym_count[r][s] as f64).collect())
        .collect();
    let (total, line, _scatter) = closed_form_rtp(&prep, &w);
    assert!(line > 0.0, "solver ways line objective must be > 0, got {line}");
    // and it must agree with the engine's exact ways RTP
    let exact = exact_rtp(&prep);
    assert!((total - exact.rtp).abs() < 1e-9, "solver ways {} != exact {}", total, exact.rtp);
}
