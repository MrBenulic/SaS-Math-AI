//! Streaming integer statistics (ADR-012). Per-thread accumulators merge associatively; all
//! money math is integer so the merge is order-independent and exactly reproducible.

use crate::accounting::rtp_ratio;

/// Win-distribution buckets, in win/bet multiples. Index 0 = zero-win; 1..=12 by the edges below.
pub const NB: usize = 14;
const EDGES: [f64; 13] = [
    0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0,
];

#[inline]
pub fn bucket_index(win: u64, bet: u64) -> usize {
    if win == 0 {
        return 0;
    }
    let m = win as f64 / bet.max(1) as f64;
    let mut idx = 1;
    for (i, e) in EDGES.iter().enumerate() {
        if m >= *e {
            idx = i + 1;
        }
    }
    idx.min(NB - 1)
}

pub fn bucket_label(i: usize) -> String {
    match i {
        0 => "0x (no win)".to_string(),
        13 => ">=5000x".to_string(),
        _ => format!("{}x-{}x", EDGES[i - 1] as u64, EDGES[i] as u64),
    }
}

/// Integer accumulator for a batch of spins.
#[derive(Debug, Clone)]
pub struct Accumulator {
    pub spins: u128,
    pub total_bet: u128,
    pub total_won: u128,
    /// Sum of win_units^2 (for variance / volatility).
    pub sum_sq_win: u128,
    /// Spins with any win.
    pub hits: u128,
    pub max_win: u64,
    pub hist: [u128; NB],
}

impl Default for Accumulator {
    fn default() -> Self {
        Accumulator {
            spins: 0,
            total_bet: 0,
            total_won: 0,
            sum_sq_win: 0,
            hits: 0,
            max_win: 0,
            hist: [0; NB],
        }
    }
}

impl Accumulator {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn record(&mut self, bet: u64, win: u64) {
        self.spins += 1;
        self.total_bet += bet as u128;
        self.total_won += win as u128;
        self.sum_sq_win += (win as u128) * (win as u128);
        if win > 0 {
            self.hits += 1;
        }
        if win > self.max_win {
            self.max_win = win;
        }
        self.hist[bucket_index(win, bet)] += 1;
    }

    pub fn merge(&mut self, o: &Accumulator) {
        self.spins += o.spins;
        self.total_bet += o.total_bet;
        self.total_won += o.total_won;
        self.sum_sq_win += o.sum_sq_win;
        self.hits += o.hits;
        if o.max_win > self.max_win {
            self.max_win = o.max_win;
        }
        for i in 0..NB {
            self.hist[i] += o.hist[i];
        }
    }

    #[inline]
    pub fn rtp(&self) -> f64 {
        rtp_ratio(self.total_won, self.total_bet)
    }

    #[inline]
    pub fn hit_frequency(&self) -> f64 {
        if self.spins == 0 {
            0.0
        } else {
            self.hits as f64 / self.spins as f64
        }
    }

    /// Variance of the per-spin return (win/bet). Volatility = sqrt of this.
    pub fn return_variance(&self, bet_units: u64) -> f64 {
        if self.spins == 0 {
            return 0.0;
        }
        let n = self.spins as f64;
        let bet = bet_units.max(1) as f64;
        let mean_win = self.total_won as f64 / n;
        let e_w2 = self.sum_sq_win as f64 / n;
        ((e_w2 - mean_win * mean_win).max(0.0)) / (bet * bet)
    }

    /// 95% confidence half-width on the RTP estimate (= 1.96 * sigma / sqrt(N)).
    pub fn rtp_ci95(&self, bet_units: u64) -> f64 {
        if self.spins == 0 {
            return 0.0;
        }
        1.96 * (self.return_variance(bet_units) / self.spins as f64).sqrt()
    }
}
