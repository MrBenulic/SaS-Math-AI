//! Auto-tuner / solver (M2, ADR-006).
//!
//! Tunes per-reel symbol weights to hit a target RTP (and optionally volatility / hit-frequency).
//! RTP is the **exact closed-form** objective (line marginals + binomial scatter) — zero-variance and
//! smooth, matching the engine's weighted-reel (independent-cell) model exactly, so re-verification
//! agrees. Volatility / hit-frequency, which need the joint distribution, are evaluated by
//! **Common Random Numbers** Monte Carlo (a fixed seed across candidates → stable comparisons).
//!
//! Non-degeneracy: each weight is bounded near its initial value (can't collapse into dead spins)
//! plus a light regularizer. Optimizer: a `(mu/mu_w, lambda)`-ES with cumulative step-size adaptation
//! (the robust core of CMA-ES). Deterministic given `seed`.
//!
//! Workflow (ADR-006): tune in weight space -> `realized_config` (integer weighted reels) -> re-verify
//! with the engine. The realized weighted-reel config is the certifiable artifact.

use crate::config::{GameConfig, Reels};
use crate::mechanics::Prepared;
use crate::simulate::monte_carlo;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};
use std::collections::HashMap;
use std::f64::consts::PI;

const LOG_BOUND: f64 = 3.0; // a weight may move exp(+-3) ~= 20x from its initial value
const EVAL_SCALE: f64 = 512.0; // realization scale for the in-loop CRN Monte Carlo

#[derive(Debug, Clone)]
pub struct SolveOptions {
    pub seed: u64,
    pub generations: usize,
    pub lambda: usize, // 0 = auto
    pub sigma0: f64,
    pub reg: f64,
    /// Spins per candidate for the CRN Monte Carlo (only used when vol/hit targets are set).
    pub eval_spins: u64,
    /// Common-random-numbers seed (fixed across candidates).
    pub crn_seed: u64,
    pub w_vol: f64,
    pub w_hit: f64,
}

impl Default for SolveOptions {
    fn default() -> Self {
        SolveOptions {
            seed: 0xC0FFEE,
            generations: 250,
            lambda: 0,
            sigma0: 0.5,
            reg: 1e-6,
            eval_spins: 40_000,
            crn_seed: 0x5EED,
            w_vol: 0.3,
            w_hit: 0.3,
        }
    }
}

/// Targets for multi-objective tuning. `volatility`/`hit_frequency` are optional.
#[derive(Debug, Clone, Copy)]
pub struct SolveTargets {
    pub rtp: f64,
    pub volatility: Option<f64>,
    pub hit_frequency: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub weights: Vec<Vec<f64>>,
    pub init_rtp: f64,
    pub line_rtp: f64,
    pub scatter_rtp: f64,
    pub total_rtp: f64,
    pub target_rtp: f64,
    pub volatility: Option<f64>,
    pub hit_frequency: Option<f64>,
    pub generations: usize,
    pub evaluations: usize,
    pub final_loss: f64,
}

// ---------- closed-form objective ----------

/// Closed-form RTP from per-reel weights: (total, line, scatter). Smooth in the weights.
pub fn closed_form_rtp(prep: &Prepared, weights: &[Vec<f64>]) -> (f64, f64, f64) {
    let probs: Vec<Vec<f64>> = (0..prep.reels)
        .map(|r| {
            let total: f64 = weights[r].iter().sum();
            if total <= 0.0 {
                vec![0.0; prep.n_symbols]
            } else {
                weights[r].iter().map(|w| w / total).collect()
            }
        })
        .collect();
    let line = line_rtp(prep, &probs);
    let scatter = scatter_rtp(prep, &probs);
    (line + scatter, line, scatter)
}

fn line_rtp(prep: &Prepared, probs: &[Vec<f64>]) -> f64 {
    let mut acc = 0.0;
    let mut tuple = vec![0usize; prep.reels];
    line_rec(prep, probs, 0, 1.0, &mut tuple, &mut acc);
    acc * prep.paylines.len() as f64 / prep.bet_units as f64
}

fn line_rec(prep: &Prepared, probs: &[Vec<f64>], reel: usize, prob: f64, tuple: &mut Vec<usize>, acc: &mut f64) {
    if reel == prep.reels {
        let pay = prep.line_pay(tuple);
        if pay > 0 {
            *acc += prob * pay as f64;
        }
        return;
    }
    for sym in 0..prep.n_symbols {
        let p = probs[reel][sym];
        if p <= 0.0 {
            continue;
        }
        tuple[reel] = sym;
        line_rec(prep, probs, reel + 1, prob * p, tuple, acc);
    }
}

fn scatter_rtp(prep: &Prepared, probs: &[Vec<f64>]) -> f64 {
    let sid = match prep.scatter_id {
        Some(s) => s,
        None => return 0.0,
    };
    let rows = prep.rows;
    let mut conv = vec![1.0f64];
    for r in 0..prep.reels {
        let p = probs[r][sid];
        let mut bin = vec![0.0; rows + 1];
        for k in 0..=rows {
            bin[k] = binom(rows, k) * p.powi(k as i32) * (1.0 - p).powi((rows - k) as i32);
        }
        let mut next = vec![0.0; conv.len() + rows];
        for (i, &ci) in conv.iter().enumerate() {
            for (j, &bj) in bin.iter().enumerate() {
                next[i + j] += ci * bj;
            }
        }
        conv = next;
    }
    let mut e = 0.0;
    for (k, &pk) in conv.iter().enumerate() {
        e += pk * prep.scatter_pay(k) as f64;
    }
    e / prep.bet_units as f64
}

fn binom(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    let mut r = 1.0;
    for i in 0..k {
        r = r * (n - i) as f64 / (i + 1) as f64;
    }
    r
}

// ---------- optimizer (shared ES core) ----------

fn seed32(seed: u64) -> [u8; 32] {
    let mut s = seed ^ 0x9E3779B97F4A7C15;
    let mut out = [0u8; 32];
    for chunk in out.chunks_mut(8) {
        s = s.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        chunk.copy_from_slice(&z.to_le_bytes());
    }
    out
}

fn gauss(rng: &mut ChaCha8Rng) -> f64 {
    let u1 = (rng.next_u64() as f64 + 1.0) / (u64::MAX as f64 + 2.0);
    let u2 = rng.next_u64() as f64 / (u64::MAX as f64 + 1.0);
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

/// `(mu/mu_w, lambda)`-ES with CSA. Minimizes `eval(x)`. Returns (best_x, best_f, evaluations).
fn run_es(n: usize, x_init: &[f64], opts: &SolveOptions, mut objective: impl FnMut(&[f64]) -> f64) -> (Vec<f64>, f64, usize) {
    let lambda = if opts.lambda > 0 { opts.lambda } else { 4 + (3.0 * (n as f64).ln()).floor() as usize };
    let mu = (lambda / 2).max(1);
    let mut w: Vec<f64> = (0..mu).map(|i| (mu as f64 + 0.5).ln() - ((i + 1) as f64).ln()).collect();
    let wsum: f64 = w.iter().sum();
    for wi in &mut w {
        *wi /= wsum;
    }
    let mu_eff = 1.0 / w.iter().map(|x| x * x).sum::<f64>();
    let nf = n as f64;
    let c_sigma = (mu_eff + 2.0) / (nf + mu_eff + 5.0);
    let d_sigma = 1.0 + 2.0 * (((mu_eff - 1.0) / (nf + 1.0)).sqrt() - 1.0).max(0.0) + c_sigma;
    let chi_n = nf.sqrt() * (1.0 - 1.0 / (4.0 * nf) + 1.0 / (21.0 * nf * nf));

    let mut m = x_init.to_vec();
    let mut p_sigma = vec![0.0; n];
    let mut sigma = opts.sigma0;
    let mut rng = ChaCha8Rng::from_seed(seed32(opts.seed));
    let mut best_x = m.clone();
    let mut best_f = f64::INFINITY;
    let mut evals = 0usize;

    for _gen in 0..opts.generations {
        let mut pop: Vec<(f64, Vec<f64>)> = Vec::with_capacity(lambda);
        for _ in 0..lambda {
            let x: Vec<f64> = (0..n).map(|i| m[i] + sigma * gauss(&mut rng)).collect();
            let f = objective(&x);
            evals += 1;
            if f < best_f {
                best_f = f;
                best_x = x.clone();
            }
            pop.push((f, x));
        }
        pop.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let m_old = m.clone();
        for i in 0..n {
            m[i] = (0..mu).map(|k| w[k] * pop[k].1[i]).sum();
        }
        let coef = (c_sigma * (2.0 - c_sigma) * mu_eff).sqrt();
        for i in 0..n {
            p_sigma[i] = (1.0 - c_sigma) * p_sigma[i] + coef * (m[i] - m_old[i]) / sigma;
        }
        let ps_norm = p_sigma.iter().map(|x| x * x).sum::<f64>().sqrt();
        sigma *= ((c_sigma / d_sigma) * (ps_norm / chi_n - 1.0)).exp();
        sigma = sigma.clamp(1e-6, 10.0);
    }
    (best_x, best_f, evals)
}

fn initial_weights(prep: &Prepared) -> Vec<Vec<f64>> {
    (0..prep.reels)
        .map(|r| (0..prep.n_symbols).map(|s| prep.sym_count[r][s] as f64).collect())
        .collect()
}

/// Build the active-parameter set and a `weights_from_x` mapping (bounded for non-degeneracy).
fn parameterize(prep: &Prepared) -> (Vec<(usize, usize)>, Vec<f64>, impl Fn(&[f64]) -> Vec<Vec<f64>> + '_) {
    let init_w = initial_weights(prep);
    let mut active: Vec<(usize, usize)> = Vec::new();
    for r in 0..prep.reels {
        for s in 0..prep.n_symbols {
            if init_w[r][s] > 0.0 {
                active.push((r, s));
            }
        }
    }
    let x_init: Vec<f64> = active.iter().map(|&(r, s)| init_w[r][s].ln()).collect();
    let active_c = active.clone();
    let x_init_c = x_init.clone();
    let n_symbols = prep.n_symbols;
    let reels = prep.reels;
    let weights_from_x = move |x: &[f64]| -> Vec<Vec<f64>> {
        let mut wv = vec![vec![0.0; n_symbols]; reels];
        for (i, &(r, s)) in active_c.iter().enumerate() {
            let lo = x_init_c[i] - LOG_BOUND;
            let hi = x_init_c[i] + LOG_BOUND;
            wv[r][s] = x[i].clamp(lo, hi).exp();
        }
        wv
    };
    (active, x_init, weights_from_x)
}

fn reg_term(x: &[f64], x_init: &[f64]) -> f64 {
    let mut r = 0.0;
    for i in 0..x.len() {
        let d = x[i] - x_init[i];
        r += d * d;
    }
    r / x.len().max(1) as f64
}

/// RTP-only tuning (pure closed-form objective; fast).
pub fn solve(prep: &Prepared, target_rtp: f64, opts: &SolveOptions) -> SolveResult {
    let (_active, x_init, weights_from_x) = parameterize(prep);
    let n = x_init.len();
    let (best_x, best_f, evals) = run_es(n, &x_init, opts, |x| {
        let wv = weights_from_x(x);
        let (total, _, _) = closed_form_rtp(prep, &wv);
        (total - target_rtp).powi(2) + opts.reg * reg_term(x, &x_init)
    });
    finalize(prep, &weights_from_x(&best_x), target_rtp, None, None, opts.generations, evals, best_f)
}

/// Multi-objective tuning: RTP (closed-form) + optional volatility / hit-frequency (CRN Monte Carlo).
pub fn solve_multi(prep: &Prepared, base: &GameConfig, targets: &SolveTargets, opts: &SolveOptions) -> SolveResult {
    let (_active, x_init, weights_from_x) = parameterize(prep);
    let n = x_init.len();
    let use_mc = targets.volatility.is_some() || targets.hit_frequency.is_some();

    let (best_x, best_f, evals) = run_es(n, &x_init, opts, |x| {
        let wv = weights_from_x(x);
        let (rtp, _, _) = closed_form_rtp(prep, &wv);
        let mut loss = ((rtp - targets.rtp) / targets.rtp).powi(2);
        if use_mc {
            let cfg2 = realized_config(base, &wv, EVAL_SCALE);
            if let Ok(p2) = Prepared::new(&cfg2) {
                let acc = monte_carlo(&p2, opts.crn_seed, opts.eval_spins);
                if let Some(tv) = targets.volatility {
                    let vol = acc.return_variance(p2.bet_units).sqrt();
                    loss += opts.w_vol * ((vol - tv) / tv).powi(2);
                }
                if let Some(th) = targets.hit_frequency {
                    let hf = acc.hit_frequency();
                    loss += opts.w_hit * ((hf - th) / th).powi(2);
                }
            }
        }
        loss + opts.reg * reg_term(x, &x_init)
    });

    let weights = weights_from_x(&best_x);
    // final measured vol/hit on the realized candidate
    let (vol, hit) = if use_mc {
        let cfg2 = realized_config(base, &weights, EVAL_SCALE);
        let p2 = Prepared::new(&cfg2).expect("prepare final");
        let acc = monte_carlo(&p2, opts.crn_seed, opts.eval_spins.max(200_000));
        (Some(acc.return_variance(p2.bet_units).sqrt()), Some(acc.hit_frequency()))
    } else {
        (None, None)
    };
    finalize(prep, &weights, targets.rtp, vol, hit, opts.generations, evals, best_f)
}

#[allow(clippy::too_many_arguments)]
fn finalize(
    prep: &Prepared,
    weights: &[Vec<f64>],
    target_rtp: f64,
    volatility: Option<f64>,
    hit_frequency: Option<f64>,
    generations: usize,
    evaluations: usize,
    final_loss: f64,
) -> SolveResult {
    let (total, line, scatter) = closed_form_rtp(prep, weights);
    let (init_total, _, _) = closed_form_rtp(prep, &initial_weights(prep));
    SolveResult {
        weights: weights.to_vec(),
        init_rtp: init_total,
        line_rtp: line,
        scatter_rtp: scatter,
        total_rtp: total,
        target_rtp,
        volatility,
        hit_frequency,
        generations,
        evaluations,
        final_loss,
    }
}

// ---------- realize + re-verify (ADR-006) ----------

/// Convert tuned float weights into integer per-reel weight maps (~`scale_target` total per reel).
pub fn weights_to_integer(weights: &[Vec<f64>], scale_target: f64) -> Vec<HashMap<usize, u64>> {
    weights
        .iter()
        .map(|wr| {
            let total: f64 = wr.iter().sum();
            let mut m = HashMap::new();
            if total > 0.0 {
                for (s, &wv) in wr.iter().enumerate() {
                    if wv > 0.0 {
                        let v = (wv / total * scale_target).round().max(1.0) as u64;
                        m.insert(s, v);
                    }
                }
            }
            m
        })
        .collect()
}

/// Produce a certifiable config from `base` with the tuned weights as integer weighted reels.
pub fn realized_config(base: &GameConfig, weights: &[Vec<f64>], scale_target: f64) -> GameConfig {
    let mut cfg = base.clone();
    cfg.reels = Reels { strips: None, weights: Some(weights_to_integer(weights, scale_target)) };
    cfg
}
