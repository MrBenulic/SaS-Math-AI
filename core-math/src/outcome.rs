//! Deterministic per-spin **outcome / event stream** — the bridge between the certified math and the
//! presentation layer (Spine skeletons, atlases, spritesheets).
//!
//! A [`RoundOutcome`] is a pure function of `(master_seed, spin_index)` (ADR-003) with integer money
//! throughout (ADR-002), so a frontend runtime can **replay** an exact spin from a seed to drive
//! animations *without re-running the math*. It shares grid construction with the fast Monte Carlo
//! path ([`crate::simulate::simulate_one`]) via `fill_grid`, and an acceptance test asserts
//! `simulate_one(..) == simulate_one_traced(..).total_win_units` over many spins so the trace can
//! never silently diverge from the certified aggregate.

use crate::mechanics::{Prepared, MAX_REELS, MAX_ROWS};
use crate::rng::{NamespacedRng, NS_BASE_REELS};
use crate::simulate::fill_grid;
use serde::Serialize;

/// Which end of the payline a winning run is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WinDirection {
    LeftToRight,
    RightToLeft,
}

/// A single winning payline run, with the exact cells a renderer should highlight/animate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LineWin {
    pub line_index: usize,
    pub symbol: usize,
    /// Run length (number of consecutive matching reels from the anchor).
    pub count: usize,
    pub direction: WinDirection,
    pub pay_units: u64,
    /// `(reel, row)` cells that form this win.
    pub cells: Vec<(usize, usize)>,
}

/// The scatter win for a spin, if any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScatterWin {
    pub symbol: usize,
    pub count: usize,
    pub pay_units: u64,
    /// Every `(reel, row)` cell holding a scatter.
    pub cells: Vec<(usize, usize)>,
}

/// A fully-resolved single spin: replayable, integer-only, deterministic from its seed coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoundOutcome {
    pub spin_index: u64,
    /// Physical reel stops (Strips model). Empty for the Weights model (independent-cell draws have
    /// no single stop — see `grid`).
    pub reel_stops: Vec<u32>,
    /// Visible `grid[reel][row]` of symbol ids — what landed on screen.
    pub grid: Vec<Vec<usize>>,
    pub line_wins: Vec<LineWin>,
    pub scatter: Option<ScatterWin>,
    /// Total win before the max-win cap (integer smallest-units).
    pub raw_win_units: u64,
    /// Certified payout after the cap (integer smallest-units).
    pub total_win_units: u64,
    /// `true` when the max-win cap clamped this spin (ADR-005).
    pub cap_applied: bool,
}

/// Fully resolve one spin into a replayable, integer-only outcome. Shares grid construction (and thus
/// RNG draw order) with [`crate::simulate::simulate_one`], so
/// `simulate_one(prep, s, i) == simulate_one_traced(prep, s, i).total_win_units`.
pub fn simulate_one_traced(prep: &Prepared, master_seed: u64, spin_index: u64) -> RoundOutcome {
    let mut rng = NamespacedRng::for_spin(master_seed, NS_BASE_REELS, spin_index);
    let mut grid = [[0usize; MAX_ROWS]; MAX_REELS];
    let (stops, has_stops) = fill_grid(prep, &mut rng, &mut grid);

    let reel_stops = if has_stops { stops[..prep.reels].to_vec() } else { Vec::new() };
    let grid_out: Vec<Vec<usize>> = (0..prep.reels)
        .map(|r| (0..prep.rows).map(|row| grid[r][row]).collect())
        .collect();

    let mut line_wins = Vec::new();
    let mut raw: u64 = 0;
    if prep.is_ways() {
        // 243-ways: whole-grid, one win per symbol; cells are every matching cell in the run's reels.
        for (wi, d) in prep.ways_wins(&grid).into_iter().enumerate() {
            raw = raw.saturating_add(d.pay);
            let s_is_wild = prep.is_wild[d.symbol];
            let mut cells: Vec<(usize, usize)> = Vec::new();
            for r in 0..d.run {
                for row in 0..prep.rows {
                    let cell = grid[r][row];
                    let m = if s_is_wild { cell == d.symbol } else { cell == d.symbol || prep.is_wild[cell] };
                    if m {
                        cells.push((r, row));
                    }
                }
            }
            line_wins.push(LineWin {
                line_index: wi,
                symbol: d.symbol,
                count: d.run,
                direction: WinDirection::LeftToRight,
                pay_units: d.pay,
                cells,
            });
        }
    } else {
        let mut line_syms = [0usize; MAX_REELS];
        for (li, line) in prep.paylines.iter().enumerate() {
            for r in 0..prep.reels {
                line_syms[r] = grid[r][line[r]];
            }
            for d in prep.line_wins(&line_syms[..prep.reels]) {
                raw = raw.saturating_add(d.pay);
                let cells: Vec<(usize, usize)> = if d.reversed {
                    // run of length `count` anchored at the rightmost reel
                    ((prep.reels - d.count)..prep.reels).map(|r| (r, line[r])).collect()
                } else {
                    (0..d.count).map(|r| (r, line[r])).collect()
                };
                line_wins.push(LineWin {
                    line_index: li,
                    symbol: d.symbol,
                    count: d.count,
                    direction: if d.reversed { WinDirection::RightToLeft } else { WinDirection::LeftToRight },
                    pay_units: d.pay,
                    cells,
                });
            }
        }
    }

    let mut scatter = None;
    if let Some(sid) = prep.scatter_id {
        let mut cells = Vec::new();
        for r in 0..prep.reels {
            for row in 0..prep.rows {
                if prep.is_scatter[grid[r][row]] {
                    cells.push((r, row));
                }
            }
        }
        let count = cells.len();
        let pay = prep.scatter_pay(count);
        if pay > 0 {
            raw = raw.saturating_add(pay);
            scatter = Some(ScatterWin { symbol: sid, count, pay_units: pay, cells });
        }
    }

    let total = raw.min(prep.cap_units);
    RoundOutcome {
        spin_index,
        reel_stops,
        grid: grid_out,
        line_wins,
        scatter,
        raw_win_units: raw,
        total_win_units: total,
        cap_applied: total < raw,
    }
}
