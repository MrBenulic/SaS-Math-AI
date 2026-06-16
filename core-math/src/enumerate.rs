//! Exact RTP (ADR-001) — the certification source of truth.
//!
//! Two layers:
//!  - **Closed-form uncapped RTP.** By linearity of expectation, line pays decompose to per-reel
//!    symbol marginals and scatter pays to a window-count convolution. Fast, exact, integer.
//!  - **Exact capped RTP (ADR-005).** The max-win cap is a non-linear clamp on the *per-spin total*,
//!    so it needs the joint outcome distribution. For the single-spin engine that space is finite:
//!    enumerate column-tuples, apply `min(win, cap)` to each atom, average. EXACT — when the space
//!    is small enough. Above `ENUM_LIMIT` we fall back to high-N Monte Carlo, **labeled numeric**
//!    (never reported as closed-form exact). Once variable-length bonus rounds exist the round-level
//!    distribution is no longer finitely enumerable and the numeric tail becomes mandatory.
//!
//! Built-in cross-check: the enumeration's *uncapped* sum must exactly equal the closed-form RTP
//! (see `enumerate_capped().uncapped_rtp`).

use crate::accounting::Rational;
use crate::mechanics::{Prepared, MAX_REELS};
use rayon::prelude::*;

/// Max number of column-tuples we will enumerate exactly. Above this, capped RTP is numeric (MC).
pub const ENUM_LIMIT: u128 = 500_000_000;

#[derive(Debug, Clone)]
pub struct ExactResult {
    /// Uncapped total RTP (closed-form marginal).
    pub rtp: f64,
    pub line_rtp: f64,
    pub scatter_rtp: f64,
    /// Exact integer ratio of the line component (handy for hand-verification).
    pub line_ratio: Rational,
    /// `Some` => exact-enumerated capped RTP. `None` => enumeration too large; use a numeric MC tail.
    pub capped: Option<f64>,
    /// Enumeration size when capped is exact; 0 when it falls back to numeric.
    pub combos: u128,
    pub note: &'static str,
}

impl ExactResult {
    /// Provenance label for the capped figure.
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
    let mut d: u128 = 1;
    for r in 0..prep.reels {
        d = d.saturating_mul(prep.strip_len[r] as u128);
    }

    // ---- closed-form uncapped line + scatter
    let mut line_acc: u128 = 0;
    let mut tuple = vec![0usize; prep.reels];
    enum_tuples(prep, 0, 1, &mut tuple, &mut line_acc);
    let line_num = line_acc.saturating_mul(prep.paylines.len() as u128);
    let scatter_num = scatter_numerator(prep);
    let den = d.saturating_mul(prep.bet_units as u128);
    let to_rtp = |num: u128| if den == 0 { 0.0 } else { num as f64 / den as f64 };

    // ---- exact capped enumeration (if feasible)
    let ce = enumerate_capped(prep);

    ExactResult {
        rtp: to_rtp(line_num.saturating_add(scatter_num)),
        line_rtp: to_rtp(line_num),
        scatter_rtp: to_rtp(scatter_num),
        line_ratio: Rational { num: line_num, den },
        capped: ce.map(|e| e.capped_rtp),
        combos: ce.map(|e| e.combos).unwrap_or(0),
        note: "uncapped = closed-form; capped = exact enumeration when feasible, else numeric MC tail",
    }
}

/// Recurse over reels building a symbol-tuple, accumulating product-of-counts * line_pay.
fn enum_tuples(prep: &Prepared, reel: usize, prod_count: u128, tuple: &mut Vec<usize>, acc: &mut u128) {
    if reel == prep.reels {
        let pay = prep.line_pay(tuple) as u128;
        if pay > 0 {
            *acc = acc.saturating_add(prod_count.saturating_mul(pay));
        }
        return;
    }
    for sym in 0..prep.n_symbols {
        let c = prep.strip_counts[reel][sym];
        if c == 0 {
            continue;
        }
        tuple[reel] = sym;
        enum_tuples(prep, reel + 1, prod_count.saturating_mul(c as u128), tuple, acc);
    }
}

/// Number of stop-combinations (out of D) weighted by scatter payout, via per-reel convolution.
fn scatter_numerator(prep: &Prepared) -> u128 {
    if prep.scatter_id.is_none() {
        return 0;
    }
    let mut conv: Vec<u128> = vec![1]; // 1 combo, 0 scatters
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
    num
}

/// Exact capped RTP via column-tuple enumeration. `None` when the space exceeds `ENUM_LIMIT`.
/// Also returns the uncapped sum from the SAME pass — it must equal the closed-form RTP.
pub fn enumerate_capped(prep: &Prepared) -> Option<CappedEnum> {
    let mut d: u128 = 1;
    for r in 0..prep.reels {
        d = d.saturating_mul(prep.strip_len[r] as u128);
    }
    if d == 0 || d > ENUM_LIMIT {
        return None;
    }
    let cap = prep.cap_units;

    // parallelize over reel 0's stops; each branch enumerates the remaining reels sequentially.
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

/// Recurse reels `reel..`, with reel 0 already pinned in `chosen`. Returns (Σcapped, Σuncapped).
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
