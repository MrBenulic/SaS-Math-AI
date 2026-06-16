//! Exact RTP by enumeration (ADR-001) — the certification source of truth.
//!
//! For a lines game, RTP decomposes exactly (linearity of expectation):
//! - **Line pays:** every payline has the same per-reel symbol marginal (strip frequency), so
//!   `E[lines] = num_paylines * sum over symbol-tuples of (prod_r count_r(s_r)) * line_pay(tuple)`.
//! - **Scatter pays:** convolve each reel's window-scatter-count distribution, then sum
//!   `count(k) * scatter_pay(k)`.
//!
//! Both share the denominator `D = product of strip lengths`. Result is an exact integer ratio.
//!
//! NOTE (M1): this is the *uncapped* theoretical RTP from line+scatter pays. The max-win cap is a
//! non-linear clamp on the per-spin total, so it is applied in the Monte Carlo engine for M1 and
//! will be folded into the exact math via joint enumeration in a later iteration (ADR-005).

use crate::accounting::Rational;
use crate::mechanics::Prepared;

#[derive(Debug, Clone)]
pub struct ExactResult {
    pub rtp: f64,
    pub rtp_ratio: Rational,
    pub line_rtp: f64,
    pub scatter_rtp: f64,
    pub note: &'static str,
}

pub fn exact_rtp(prep: &Prepared) -> ExactResult {
    let mut d: u128 = 1;
    for r in 0..prep.reels {
        d = d.saturating_mul(prep.strip_len[r] as u128);
    }

    // ---- line numerator
    let mut line_acc: u128 = 0;
    let mut tuple = vec![0usize; prep.reels];
    enum_tuples(prep, 0, 1, &mut tuple, &mut line_acc);
    let line_num = line_acc.saturating_mul(prep.paylines.len() as u128);

    // ---- scatter numerator
    let scatter_num = scatter_numerator(prep);

    let total_num = line_num.saturating_add(scatter_num);
    let den = d.saturating_mul(prep.bet_units as u128);

    let to_rtp = |num: u128| if den == 0 { 0.0 } else { num as f64 / den as f64 };
    ExactResult {
        rtp: to_rtp(total_num),
        rtp_ratio: Rational { num: total_num, den },
        line_rtp: to_rtp(line_num),
        scatter_rtp: to_rtp(scatter_num),
        note: "exact line+scatter RTP, uncapped (max-win cap applied in Monte Carlo for M1)",
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

/// Number of stop-combinations (out of D) weighted by scatter payout, via convolution.
fn scatter_numerator(prep: &Prepared) -> u128 {
    if prep.scatter_id.is_none() {
        return 0;
    }
    // conv[k] = number of stop-combinations so far yielding total k scatters.
    let mut conv: Vec<u128> = vec![1]; // 1 combo, 0 scatters
    for r in 0..prep.reels {
        let len = prep.strip_len[r] as usize;
        let strip = &prep.strips[r];
        let mut per = vec![0u128; prep.rows + 1];
        for stop in 0..len {
            let mut k = 0usize;
            for row in 0..prep.rows {
                if prep.is_scatter[strip[(stop + row) % len]] {
                    k += 1;
                }
            }
            per[k] += 1;
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
        if cnt == 0 {
            continue;
        }
        let pay = prep.scatter_pay(k) as u128;
        if pay > 0 {
            num = num.saturating_add(cnt.saturating_mul(pay));
        }
    }
    num
}
