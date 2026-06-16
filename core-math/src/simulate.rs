//! Deterministic Monte Carlo (ADR-002, ADR-003). Each spin is seeded only by
//! `(master_seed, spin_index)`, so results are identical regardless of thread scheduling. Integer
//! accounting makes the parallel reduction exactly reproducible.
//!
//! Strips model: draw one stop per reel, read the consecutive window. Weights model: draw each
//! visible cell independently from the reel's weight distribution (matches the binomial exact math).

use crate::mechanics::{Prepared, ReelModel, MAX_REELS, MAX_ROWS};
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

    // build the visible grid[reel][row]
    let mut grid = [[0usize; MAX_ROWS]; MAX_REELS];
    match prep.model {
        ReelModel::Strips => {
            for r in 0..prep.reels {
                let len = prep.reel_size[r] as usize;
                let stop = rng.below(prep.reel_size[r] as u32) as usize;
                for row in 0..prep.rows {
                    grid[r][row] = prep.strips[r][(stop + row) % len];
                }
            }
        }
        ReelModel::Weights => {
            for r in 0..prep.reels {
                let size = prep.reel_size[r] as u32;
                for row in 0..prep.rows {
                    let x = rng.below(size) as u64;
                    grid[r][row] = prep.sample_weighted(r, x);
                }
            }
        }
    }

    let mut total: u64 = 0;
    let mut line_syms = [0usize; MAX_REELS];
    for line in &prep.paylines {
        for r in 0..prep.reels {
            line_syms[r] = grid[r][line[r]];
        }
        total = total.saturating_add(prep.line_pay(&line_syms[..prep.reels]));
    }

    if prep.scatter_id.is_some() {
        let mut sc = 0usize;
        for r in 0..prep.reels {
            for row in 0..prep.rows {
                if prep.is_scatter[grid[r][row]] {
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

/// Same as `monte_carlo`, pinned to a fixed-size Rayon thread pool (proves thread-independence).
pub fn monte_carlo_threads(prep: &Prepared, master_seed: u64, spins: u64, threads: usize) -> Accumulator {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("build rayon pool");
    pool.install(|| monte_carlo(prep, master_seed, spins))
}
