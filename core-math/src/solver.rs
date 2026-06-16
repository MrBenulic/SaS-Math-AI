//! Auto-tuner / solver (M2, ADR-006).
//!
//! Tunes per-reel symbol weights to hit a target RTP. The objective is the **exact closed-form
//! RTP** as a function of the weights (line pays via per-reel symbol marginals; scatter pays via a
//! binomial window model) — zero-variance and smooth, so the optimizer never fights Monte Carlo
//! noise. Non-degeneracy is enforced by **bounding** each weight near its initial value (you cannot
//! collapse the game into dead spins) plus a light regularizer toward the start.
//!
//! Optimizer: a (mu/mu_w, lambda)-Evolution Strategy with cumulative step-size adaptation (CSA) —
//! the robust core of CMA-ES (full covariance adaptation is the planned upgrade). Deterministic
//! given `seed`.
//!
//! Workflow (ADR-006): tune in weight space -> realize integer strips -> re-verify with the exact
//! integer engine + Monte Carlo. `realized_config` produces the certifiable artifact.

use crate::config::{GameConfig, Reels};
use crate::mechanics::Prepared;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};
use std::collections::HashMap;
use std::f64::consts::PI;

/// How far (in log-weight space) a weight may move from its initial value. exp(3) ~= 20x.
const LOG_BOUND: f64 = 3.0;

#[derive(Debug, Clone)]
pub struct SolveOptions {
    pub seed: u64,
    pub generations: usize,
    /// Population size; 0 = auto (4 + floor(3 ln N)).
    pub lambda: usize,
    /// Initial step size in log-weight space.
    pub sigma0: f64,
    /// Regularizer toward the initial weights (tie-breaker; keeps the game's shape).
    pub reg: f64,
}

impl Default for SolveOptions {
    fn default() -> Self {
        SolveOptions { seed: 0xC0FFEE, generations: 250, lambda: 0, sigma0: 0.5, reg: 1e-6 }
    }
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    /// Tuned per-reel weights, indexed [reel][symbol_id] (0.0 for symbols absent on that reel).
    pub weights: Vec<Vec<f64>>,
    pub init_rtp: f64,
    pub line_rtp: f64,
    pub scatter_rtp: f64,
    pub total_rtp: f64,
    pub target_rtp: f64,
    pub generations: usize,
    pub evaluations: usize,
    pub final_loss: f64,
}

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

/// Scatter RTP via an independent-cell binomial window model (closed-form in the weights).
fn scatter_rtp(prep: &Prepared, probs: &[Vec<f64>]) -> f64 {
    let sid = match prep.scatter_id {
        Some(s) => s,
        None => return 0.0,
    };
    let rows = prep.rows;
    let mut conv = vec![1.0f64]; // P(0 scatters) = 1
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

// ---------- optimizer ----------

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

/// Initial weights from the prepared model: the realized strip's per-reel symbol frequencies.
fn initial_weights(prep: &Prepared) -> Vec<Vec<f64>> {
    (0..prep.reels)
        .map(|r| (0..prep.n_symbols).map(|s| prep.strip_counts[r][s] as f64).collect())
        .collect()
}

pub fn solve(prep: &Prepared, target_rtp: f64, opts: &SolveOptions) -> SolveResult {
    let init_w = initial_weights(prep);

    // active parameters = (reel, symbol) with a positive initial weight
    let mut active: Vec<(usize, usize)> = Vec::new();
    for r in 0..prep.reels {
        for s in 0..prep.n_symbols {
            if init_w[r][s] > 0.0 {
                active.push((r, s));
            }
        }
    }
    let n = active.len();
    let x_init: Vec<f64> = active.iter().map(|&(r, s)| init_w[r][s].ln()).collect();

    // build weights from a parameter vector (clamped to bounds for non-degeneracy)
    let weights_from_x = |x: &[f64]| -> Vec<Vec<f64>> {
        let mut w = vec![vec![0.0; prep.n_symbols]; prep.reels];
        for (i, &(r, s)) in active.iter().enumerate() {
            let lo = x_init[i] - LOG_BOUND;
            let hi = x_init[i] + LOG_BOUND;
            w[r][s] = x[i].clamp(lo, hi).exp();
        }
        w
    };
    let loss = |x: &[f64], w: &[Vec<f64>]| -> f64 {
        let (total, _, _) = closed_form_rtp(prep, w);
        let mut reg = 0.0;
        for i in 0..n {
            let d = x[i] - x_init[i];
            reg += d * d;
        }
        (total - target_rtp).powi(2) + opts.reg * reg / n.max(1) as f64
    };

    // ES constants
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

    let mut m = x_init.clone();
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
            let wts = weights_from_x(&x);
            let f = loss(&x, &wts);
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
        // CSA step-size update (isotropic: C = I)
        let coef = (c_sigma * (2.0 - c_sigma) * mu_eff).sqrt();
        for i in 0..n {
            let zw = (m[i] - m_old[i]) / sigma;
            p_sigma[i] = (1.0 - c_sigma) * p_sigma[i] + coef * zw;
        }
        let ps_norm = p_sigma.iter().map(|x| x * x).sum::<f64>().sqrt();
        sigma *= ((c_sigma / d_sigma) * (ps_norm / chi_n - 1.0)).exp();
        sigma = sigma.clamp(1e-6, 10.0);
    }

    let weights = weights_from_x(&best_x);
    let (total, line, scatter) = closed_form_rtp(prep, &weights);
    let (init_total, _, _) = closed_form_rtp(prep, &init_w);
    SolveResult {
        weights,
        init_rtp: init_total,
        line_rtp: line,
        scatter_rtp: scatter,
        total_rtp: total,
        target_rtp,
        generations: opts.generations,
        evaluations: evals,
        final_loss: best_f,
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

/// Produce a certifiable config from `base` with the tuned weights as integer weight-reels.
pub fn realized_config(base: &GameConfig, weights: &[Vec<f64>], scale_target: f64) -> GameConfig {
    let mut cfg = base.clone();
    cfg.reels = Reels { strips: None, weights: Some(weights_to_integer(weights, scale_target)) };
    cfg
}
