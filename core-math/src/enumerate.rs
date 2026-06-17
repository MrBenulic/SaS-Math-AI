//! Exact RTP (ADR-001) — the certification source of truth.
//!
//! - **Line pays:** per-reel symbol marginal (`sym_count / reel_size`), summed over symbol-tuples.
//!   Integer-exact for both reel models.
//! - **Scatter pays:** Strips -> positional window-count convolution (integer); Weights ->
//!   independent-cell binomial-per-reel convolution (f64, the model's exact value).
//! - **Capped RTP (ADR-005):** Strips only — enumerate the finite column-tuple space, `min(win,cap)`
//!   per atom, average. `None` for Weights (independent-cell space is not finitely enumerable -> the
//!   capped figure comes from the numeric MC tail). Cross-check: enumeration-uncapped == closed-form.

use crate::accounting::Rational;
use crate::mechanics::{Prepared, ReelModel, MAX_REELS};
use rayon::prelude::*;

pub const ENUM_LIMIT: u128 = 500_000_000;

#[derive(Debug, Clone)]
pub struct ExactResult {
    pub rtp: f64,
    pub line_rtp: f64,
    pub scatter_rtp: f64,
    pub line_ratio: Rational,
    pub capped: Option<f64>,
    pub combos: u128,
    pub note: &'static str,
}

impl ExactResult {
    pub fn capped_provenance(&self) -> &'static str {
        if self.capped.is_some() {
            "exact-enumerated"
        } else {
            "numeric-tail (Monte Carlo)"
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CappedEnum {
    pub capped_rtp: f64,
    pub uncapped_rtp: f64,
    pub combos: u128,
}

pub fn exact_rtp(prep: &Prepared) -> ExactResult {
    if prep.is_ways() {
        let ways = ways_rtp_exact(prep);
        let scatter_rtp = scatter_rtp_exact(prep);
        return ExactResult {
            rtp: ways + scatter_rtp,
            line_rtp: ways,
            scatter_rtp,
            // Ways RTP is a closed-form expectation (product of per-reel means) — not a single integer
            // ratio — so the rational is left empty (0/0); the f64 above is the exact theoretical value.
            line_ratio: Rational { num: 0, den: 0 },
            capped: None,
            combos: 0,
            note: "ways(243) = exact closed-form over per-reel count distributions; scatter as usual; capped via numeric MC tail",
        };
    }
    let (line_num, line_den) = line_exact(prep);
    let line_rtp = if line_den == 0 { 0.0 } else { line_num as f64 / line_den as f64 };
    let scatter_rtp = scatter_rtp_exact(prep);

    let ce = enumerate_capped(prep);

    ExactResult {
        rtp: line_rtp + scatter_rtp,
        line_rtp,
        scatter_rtp,
        line_ratio: Rational { num: line_num, den: line_den },
        capped: ce.map(|e| e.capped_rtp),
        combos: ce.map(|e| e.combos).unwrap_or(0),
        note: "line = integer marginal; scatter = window(Strips)/binomial(Weights); capped exact when enumerable, else numeric MC",
    }
}

/// Exact line RTP as an integer ratio (num, den). Uses the per-reel symbol marginal for both models.
fn line_exact(prep: &Prepared) -> (u128, u128) {
    let mut d: u128 = 1;
    for r in 0..prep.reels {
        d = d.saturating_mul(prep.reel_size[r] as u128);
    }
    let mut acc: u128 = 0;
    let mut tuple = vec![0usize; prep.reels];
    line_enum(prep, 0, 1, &mut tuple, &mut acc);
    let num = acc.saturating_mul(prep.paylines.len() as u128);
    (num, d.saturating_mul(prep.bet_units as u128))
}

fn line_enum(prep: &Prepared, reel: usize, prod: u128, tuple: &mut Vec<usize>, acc: &mut u128) {
    if reel == prep.reels {
        let pay = prep.line_pay(tuple) as u128;
        if pay > 0 {
            *acc = acc.saturating_add(prod.saturating_mul(pay));
        }
        return;
    }
    for sym in 0..prep.n_symbols {
        let c = prep.sym_count[reel][sym];
        if c == 0 {
            continue;
        }
        tuple[reel] = sym;
        line_enum(prep, reel + 1, prod.saturating_mul(c as u128), tuple, acc);
    }
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

/// Per-reel stats for a 243-ways symbol: `(mean count of s-or-wild, P(count == 0))` over reel `r`'s
/// visible window. Strips -> empirical over all stops; Weights -> Binomial(rows, p).
fn ways_reel_stats(prep: &Prepared, r: usize, s: usize) -> (f64, f64) {
    let s_is_wild = prep.is_wild[s];
    match prep.model {
        ReelModel::Strips => {
            let cols = &prep.columns[r];
            let n = cols.len() as f64;
            let (mut sum, mut zero) = (0.0f64, 0.0f64);
            for col in cols {
                let mut c = 0u32;
                for row in 0..prep.rows {
                    let cell = col.syms[row];
                    let m = if s_is_wild { cell == s } else { cell == s || prep.is_wild[cell] };
                    if m {
                        c += 1;
                    }
                }
                sum += c as f64;
                if c == 0 {
                    zero += 1.0;
                }
            }
            if n == 0.0 { (0.0, 1.0) } else { (sum / n, zero / n) }
        }
        ReelModel::Weights => {
            let rs = prep.reel_size[r] as f64;
            if rs <= 0.0 {
                return (0.0, 1.0);
            }
            let p = if s_is_wild {
                prep.sym_count[r][s] as f64 / rs
            } else {
                let wild_w: u64 = (0..prep.n_symbols)
                    .filter(|&w| prep.is_wild[w])
                    .map(|w| prep.sym_count[r][w])
                    .sum();
                (prep.sym_count[r][s] + wild_w) as f64 / rs
            };
            (prep.rows as f64 * p, (1.0 - p).powi(prep.rows as i32))
        }
    }
}

/// Exact 243-ways RTP as a closed-form expectation: for each non-scatter symbol, the run pays
/// `pays[s][K] * (product of per-reel mean counts over reels 0..K)`, weighted by `P(count==0)` at the
/// run-ending reel. Reels are independent (per-reel stop / independent cells), so the expectation
/// factorizes. Zero sampling error; rendered in f64.
fn ways_rtp_exact(prep: &Prepared) -> f64 {
    let bet = prep.bet_units as f64;
    if bet <= 0.0 {
        return 0.0;
    }
    let reels = prep.reels;
    let mut total = 0.0f64;
    for s in 0..prep.n_symbols {
        if prep.is_scatter[s] {
            continue;
        }
        let stats: Vec<(f64, f64)> = (0..reels).map(|r| ways_reel_stats(prep, r, s)).collect();
        let mut prod = 1.0f64; // product of mean counts over reels 0..K-1
        for k in 1..=reels {
            prod *= stats[k - 1].0;
            let pay = if k <= prep.max_count { prep.pays[s][k] } else { 0 };
            if pay > 0 {
                let tail = if k < reels { stats[k].1 } else { 1.0 }; // P(run ends after reel K-1)
                total += pay as f64 * prod * tail;
            }
        }
    }
    total / bet
}

fn scatter_rtp_exact(prep: &Prepared) -> f64 {
    let sid = match prep.scatter_id {
        Some(s) => s,
        None => return 0.0,
    };
    let bet = prep.bet_units as f64;
    match prep.model {
        ReelModel::Strips => {
            let mut d: u128 = 1;
            for r in 0..prep.reels {
                d = d.saturating_mul(prep.reel_size[r] as u128);
            }
            let mut conv: Vec<u128> = vec![1];
            for r in 0..prep.reels {
                let mut per = vec![0u128; prep.rows + 1];
                for col in &prep.columns[r] {
                    per[col.scatters as usize] += 1;
                }
                let mut next = vec![0u128; conv.len() + prep.rows];
                for (i, &ci) in conv.iter().enumerate() {
                    if ci == 0 {
                        continue;
                    }
                    for (j, &pj) in per.iter().enumerate() {
                        if pj == 0 {
                            continue;
                        }
                        next[i + j] = next[i + j].saturating_add(ci.saturating_mul(pj));
                    }
                }
                conv = next;
            }
            let mut num = 0u128;
            for (k, &cnt) in conv.iter().enumerate() {
                if cnt > 0 {
                    num = num.saturating_add(cnt.saturating_mul(prep.scatter_pay(k) as u128));
                }
            }
            if d == 0 { 0.0 } else { num as f64 / (d as f64 * bet) }
        }
        ReelModel::Weights => {
            // independent-cell: per reel scatter count ~ Binomial(rows, p), convolved across reels
            let mut conv = vec![1.0f64];
            for r in 0..prep.reels {
                let p = prep.sym_count[r][sid] as f64 / prep.reel_size[r] as f64;
                let mut bin = vec![0.0; prep.rows + 1];
                for k in 0..=prep.rows {
                    bin[k] = binom(prep.rows, k) * p.powi(k as i32) * (1.0 - p).powi((prep.rows - k) as i32);
                }
                let mut next = vec![0.0; conv.len() + prep.rows];
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
            e / bet
        }
    }
}

/// Exact capped RTP via column-tuple enumeration (Strips only). `None` for Weights or when the
/// space exceeds `ENUM_LIMIT`.
pub fn enumerate_capped(prep: &Prepared) -> Option<CappedEnum> {
    if prep.model != ReelModel::Strips || prep.is_ways() {
        // The capped walk is payline-based; ways games use the whole-grid `ways_pay` instead.
        return None;
    }
    let mut d: u128 = 1;
    for r in 0..prep.reels {
        d = d.saturating_mul(prep.reel_size[r] as u128);
    }
    if d == 0 || d > ENUM_LIMIT {
        return None;
    }
    let cap = prep.cap_units;
    let (sum_c, sum_u): (u128, u128) = prep.columns[0]
        .par_iter()
        .enumerate()
        .map(|(i0, c0)| {
            let mut chosen = [0usize; MAX_REELS];
            chosen[0] = i0;
            rec_capped(prep, 1, &mut chosen, c0.scatters, cap)
        })
        .reduce(|| (0u128, 0u128), |a, b| (a.0 + b.0, a.1 + b.1));

    let den = d.saturating_mul(prep.bet_units as u128);
    Some(CappedEnum {
        capped_rtp: if den == 0 { 0.0 } else { sum_c as f64 / den as f64 },
        uncapped_rtp: if den == 0 { 0.0 } else { sum_u as f64 / den as f64 },
        combos: d,
    })
}

fn rec_capped(prep: &Prepared, reel: usize, chosen: &mut [usize; MAX_REELS], scat: u32, cap: u64) -> (u128, u128) {
    if reel == prep.reels {
        let mut total: u64 = 0;
        let mut line_syms = [0usize; MAX_REELS];
        for line in &prep.paylines {
            for r in 0..prep.reels {
                line_syms[r] = prep.columns[r][chosen[r]].syms[line[r]];
            }
            total = total.saturating_add(prep.line_pay(&line_syms[..prep.reels]));
        }
        total = total.saturating_add(prep.scatter_pay(scat as usize));
        return (total.min(cap) as u128, total as u128);
    }
    let mut sc = 0u128;
    let mut su = 0u128;
    for (idx, col) in prep.columns[reel].iter().enumerate() {
        chosen[reel] = idx;
        let (a, b) = rec_capped(prep, reel + 1, chosen, scat + col.scatters, cap);
        sc += a;
        su += b;
    }
    (sc, su)
}
