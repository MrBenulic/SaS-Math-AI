//! Line / scatter pay "Lego blocks". The SAME payout semantics are used by both the exact
//! engine and the Monte Carlo engine, so any agreement between them is meaningful.
//!
//! `Prepared` precomputes flat lookup tables from a `GameConfig` for allocation-free evaluation.

use crate::config::{GameConfig, LineEval, SymbolKind};

pub const MAX_REELS: usize = 16;

pub struct Prepared {
    pub reels: usize,
    pub rows: usize,
    pub bet_units: u64,
    /// floor(cap_x * bet_units); `u64::MAX` when effectively non-binding.
    pub cap_units: u64,
    pub line_eval: LineEval,
    pub paylines: Vec<Vec<usize>>,
    pub strips: Vec<Vec<usize>>,
    pub strip_len: Vec<u32>,
    pub n_symbols: usize,
    pub max_count: usize, // reels * rows (max scatter count)
    pub is_wild: Vec<bool>,
    pub is_scatter: Vec<bool>,
    /// pays[symbol_id][count] -> payout units (0 if undefined). count in 0..=max_count.
    pub pays: Vec<Vec<u64>>,
    pub scatter_id: Option<usize>,
    /// strip_counts[reel][symbol_id] -> occurrences on that reel's strip.
    pub strip_counts: Vec<Vec<u64>>,
}

impl Prepared {
    pub fn new(cfg: &GameConfig) -> Result<Prepared, String> {
        let reels = cfg.game.grid.reels;
        let rows = cfg.game.grid.rows;
        if reels == 0 || reels > MAX_REELS {
            return Err(format!("reels must be 1..={MAX_REELS}, got {reels}"));
        }
        if cfg.reels.strips.len() != reels {
            return Err(format!(
                "reels.strips has {} reels but grid.reels = {reels}",
                cfg.reels.strips.len()
            ));
        }
        for (r, line) in cfg.paylines.iter().enumerate() {
            if line.len() != reels {
                return Err(format!("payline {r} has {} cells, expected {reels}", line.len()));
            }
            if line.iter().any(|&row| row >= rows) {
                return Err(format!("payline {r} references a row >= rows ({rows})"));
            }
        }

        let n_symbols = cfg.symbols.iter().map(|s| s.id).max().map(|m| m + 1).unwrap_or(0);
        let max_count = reels * rows;

        let mut is_wild = vec![false; n_symbols];
        let mut is_scatter = vec![false; n_symbols];
        let mut scatter_id = None;
        for s in &cfg.symbols {
            match s.kind {
                SymbolKind::Wild => is_wild[s.id] = true,
                SymbolKind::Scatter => {
                    is_scatter[s.id] = true;
                    scatter_id = Some(s.id);
                }
                SymbolKind::Regular => {}
            }
        }

        let mut pays = vec![vec![0u64; max_count + 1]; n_symbols];
        for e in &cfg.paytable {
            if e.symbol_id >= n_symbols {
                return Err(format!("paytable references unknown symbol_id {}", e.symbol_id));
            }
            for (&count, &payout) in &e.pays {
                let c = count as usize;
                if c <= max_count {
                    pays[e.symbol_id][c] = payout;
                }
            }
        }

        let mut strip_counts = vec![vec![0u64; n_symbols]; reels];
        let mut strip_len = vec![0u32; reels];
        for (r, strip) in cfg.reels.strips.iter().enumerate() {
            if strip.is_empty() {
                return Err(format!("reel {r} strip is empty"));
            }
            strip_len[r] = strip.len() as u32;
            for &sym in strip {
                if sym >= n_symbols {
                    return Err(format!("reel {r} references unknown symbol id {sym}"));
                }
                strip_counts[r][sym] += 1;
            }
        }

        let cap_units = {
            let c = cfg.max_win.cap_x * cfg.game.bet_units as f64;
            if c.is_finite() && c < u64::MAX as f64 {
                c.floor() as u64
            } else {
                u64::MAX
            }
        };

        Ok(Prepared {
            reels,
            rows,
            bet_units: cfg.game.bet_units,
            cap_units,
            line_eval: cfg.line_eval,
            paylines: cfg.paylines.clone(),
            strips: cfg.reels.strips.clone(),
            strip_len,
            n_symbols,
            max_count,
            is_wild,
            is_scatter,
            pays,
            scatter_id,
            strip_counts,
        })
    }

    /// Leading run length from reel 0 for candidate paying symbol `sym` (wilds substitute).
    #[inline]
    fn run_len(&self, sym: usize, line: &[usize]) -> usize {
        let sym_is_wild = self.is_wild[sym];
        let mut run = 0;
        for &s in line {
            let matches = if sym_is_wild {
                s == sym
            } else {
                s == sym || self.is_wild[s]
            };
            if matches {
                run += 1;
            } else {
                break;
            }
        }
        run
    }

    #[inline]
    fn run_pay(&self, sym: usize, line: &[usize]) -> u64 {
        let r = self.run_len(sym, line);
        if r >= 1 && r <= self.max_count {
            self.pays[sym][r]
        } else {
            0
        }
    }

    /// Payout (units) for one payline given the symbol on each reel, left-to-right.
    fn line_pay_ltr(&self, line: &[usize]) -> u64 {
        if line.is_empty() || self.is_scatter[line[0]] {
            return 0;
        }
        if self.is_wild[line[0]] {
            // leading wild can resolve to any paying symbol: take the best.
            let mut best = 0;
            for sym in 0..self.n_symbols {
                if self.is_scatter[sym] {
                    continue;
                }
                let p = self.run_pay(sym, line);
                if p > best {
                    best = p;
                }
            }
            best
        } else {
            // a run can only start with this exact regular symbol.
            self.run_pay(line[0], line)
        }
    }

    /// Public per-line payout honoring `line_eval`.
    #[inline]
    pub fn line_pay(&self, line: &[usize]) -> u64 {
        match self.line_eval {
            LineEval::LeftToRight => self.line_pay_ltr(line),
            LineEval::BothWays => {
                let l = self.line_pay_ltr(line);
                let mut rev: [usize; MAX_REELS] = [0; MAX_REELS];
                let n = line.len();
                for i in 0..n {
                    rev[i] = line[n - 1 - i];
                }
                l + self.line_pay_ltr(&rev[..n])
            }
            LineEval::Ways243 => 0, // not supported in M1
        }
    }

    /// Scatter payout for `count` scatters anywhere in the window (count clamped to the table).
    #[inline]
    pub fn scatter_pay(&self, count: usize) -> u64 {
        match self.scatter_id {
            Some(id) => self.pays[id][count.min(self.max_count)],
            None => 0,
        }
    }
}
