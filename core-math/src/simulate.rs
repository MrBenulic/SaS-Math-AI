//! Deterministic Monte Carlo (ADR-002, ADR-003). Each spin is seeded only by
//! `(master_seed, spin_index)`, so results are identical regardless of how Rayon schedules
//! threads. Integer accounting makes the parallel reduction exactly reproducible.

use crate::mechanics::{Prepared, MAX_REELS};
use crate::rng::{NamespacedRng, NS_BASE_REELS};
use crate::stats::Accumulator;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct McResult {
    pub acc: Accumulator,
    pub spins: u64,
}

/// One spin's total win in integer units (capped). Pure function of `(master_seed, spin_index)`.
#[inline]
pub fn simulate_one(prep: &Prepared, master_seed: u64, spin_index: u64) -> u64 {
    let mut rng = NamespacedRng::for_spin(master_seed, NS_BASE_REELS, spin_index);

    let mut stops = [0u32; MAX_REELS];
    for r in 0..prep.reels {
        stops[r] = rng.below(prep.strip_len[r]);
    }

    let mut total: u64 = 0;

    // line wins
    let mut line_syms = [0usize; MAX_REELS];
    for line in &prep.paylines {
        for r in 0..prep.reels {
            let len = prep.strip_len[r] as usize;
            let pos = (stops[r] as usize + line[r]) % len;
            line_syms[r] = prep.strips[r][pos];
        }
        total = total.saturating_add(prep.line_pay(&line_syms[..prep.reels]));
    }

    // scatter wins (count over the whole visible window)
    if prep.scatter_id.is_some() {
        let mut sc = 0usize;
        for r in 0..prep.reels {
            let len = prep.strip_len[r] as usize;
            for row in 0..prep.rows {
                let pos = (stops[r] as usize + row) % len;
                if prep.is_scatter[prep.strips[r][pos]] {
                    sc += 1;
                }
            }
        }
        total = total.saturating_add(prep.scatter_pay(sc));
    }

    total.min(prep.cap_units)
}

/// Run `spins` deterministic Monte Carlo spins in parallel and return the merged accumulator.
pub fn monte_carlo(prep: &Prepared, master_seed: u64, spins: u64) -> Accumulator {
    let bet = prep.bet_units;
    (0..spins)
        .into_par_iter()
        .fold(Accumulator::new, |mut acc, spin_index| {
            acc.record(bet, simulate_one(prep, master_seed, spin_index));
            acc
        })
        .reduce(Accumulator::new, |mut a, b| {
            a.merge(&b);
            a
        })
}

/// Same as `monte_carlo`, pinned to a fixed-size Rayon thread pool. Used to prove that the result
/// is identical regardless of thread count (determinism / thread-independence).
pub fn monte_carlo_threads(prep: &Prepared, master_seed: u64, spins: u64, threads: usize) -> Accumulator {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("build rayon pool");
    pool.install(|| monte_carlo(prep, master_seed, spins))
}
