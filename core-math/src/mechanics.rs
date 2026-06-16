//! Line / scatter pay "Lego blocks" + the prepared engine model.
//!
//! Two reel models share the same payout semantics (`line_pay` / `scatter_pay`):
//!  - **Strips** (physical reels): the visible window is `rows` consecutive strip positions, so
//!    scatter counting is positional and the per-spin outcome space is finitely enumerable.
//!  - **Weights** (virtual reels): each visible cell is an independent weighted draw, so scatter
//!    follows a binomial-per-reel model. This is the solver's tuning representation (ADR-006); the
//!    solver's closed-form objective and the engine's re-verification then use the SAME model.
//!
//! Either way the per-reel symbol marginal (used for line pays) is `sym_count / reel_size`.

use crate::config::{GameConfig, LineEval, SymbolKind};

pub const MAX_REELS: usize = 16;
pub const MAX_ROWS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReelModel {
    /// Physical strips: positional window scatter; finite outcome space.
    Strips,
    /// Virtual weighted reels: independent-cell draws; binomial scatter.
    Weights,
}

/// One reel-stop's visible column (Strips model only): symbol per row + scatter count.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub syms: [usize; MAX_ROWS],
    pub scatters: u32,
}

pub struct Prepared {
    pub reels: usize,
    pub rows: usize,
    pub bet_units: u64,
    pub cap_units: u64,
    pub line_eval: LineEval,
    pub paylines: Vec<Vec<usize>>,
    pub n_symbols: usize,
    pub max_count: usize,
    pub is_wild: Vec<bool>,
    pub is_scatter: Vec<bool>,
    pub pays: Vec<Vec<u64>>,
    pub scatter_id: Option<usize>,

    pub model: ReelModel,
    /// Per-reel symbol "count": strip occurrences (Strips) or weight (Weights). Drives line marginals.
    pub sym_count: Vec<Vec<u64>>,
    /// Per-reel total: strip length (Strips) or sum of weights (Weights).
    pub reel_size: Vec<u64>,

    // ---- Strips model only (empty for Weights) ----
    pub strips: Vec<Vec<usize>>,
    pub columns: Vec<Vec<ColumnInfo>>,

    // ---- Weights model only (empty for Strips) ----
    /// Per reel: symbol ids with positive weight, and their cumulative weights (for MC sampling).
    pub wsyms: Vec<Vec<usize>>,
    pub wcum: Vec<Vec<u64>>,
}

impl Prepared {
    pub fn is_weighted(&self) -> bool {
        self.model == ReelModel::Weights
    }

    pub fn new(cfg: &GameConfig) -> Result<Prepared, String> {
        let reels = cfg.game.grid.reels;
        let rows = cfg.game.grid.rows;
        if reels == 0 || reels > MAX_REELS {
            return Err(format!("reels must be 1..={MAX_REELS}, got {reels}"));
        }
        if rows == 0 || rows > MAX_ROWS {
            return Err(format!("rows must be 1..={MAX_ROWS}, got {rows}"));
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

        // Resolve the reel model.
        let model;
        let mut sym_count = vec![vec![0u64; n_symbols]; reels];
        let mut reel_size = vec![0u64; reels];
        let mut strips: Vec<Vec<usize>> = Vec::new();
        let mut columns: Vec<Vec<ColumnInfo>> = Vec::new();
        let mut wsyms: Vec<Vec<usize>> = Vec::new();
        let mut wcum: Vec<Vec<u64>> = Vec::new();

        match (&cfg.reels.strips, &cfg.reels.weights) {
            (Some(s), _) => {
                model = ReelModel::Strips;
                if s.len() != reels {
                    return Err(format!("reels.strips has {} reels but grid.reels = {reels}", s.len()));
                }
                strips = s.clone();
                for (r, strip) in strips.iter().enumerate() {
                    if strip.is_empty() {
                        return Err(format!("reel {r} strip is empty"));
                    }
                    reel_size[r] = strip.len() as u64;
                    for &sym in strip {
                        if sym >= n_symbols {
                            return Err(format!("reel {r} references unknown symbol id {sym}"));
                        }
                        sym_count[r][sym] += 1;
                    }
                    let len = strip.len();
                    let mut reel_cols = Vec::with_capacity(len);
                    for stop in 0..len {
                        let mut syms = [0usize; MAX_ROWS];
                        let mut scatters = 0u32;
                        for row in 0..rows {
                            let sym = strip[(stop + row) % len];
                            syms[row] = sym;
                            if is_scatter[sym] {
                                scatters += 1;
                            }
                        }
                        reel_cols.push(ColumnInfo { syms, scatters });
                    }
                    columns.push(reel_cols);
                }
            }
            (None, Some(w)) => {
                model = ReelModel::Weights;
                if w.len() != reels {
                    return Err(format!("reels.weights has {} reels but grid.reels = {reels}", w.len()));
                }
                wsyms = vec![Vec::new(); reels];
                wcum = vec![Vec::new(); reels];
                for (r, wmap) in w.iter().enumerate() {
                    let mut ids: Vec<usize> = wmap.keys().copied().filter(|s| wmap[s] > 0).collect();
                    ids.sort_unstable();
                    if ids.is_empty() {
                        return Err(format!("reel {r} has no positive weights"));
                    }
                    let mut cum = 0u64;
                    for &sym in &ids {
                        if sym >= n_symbols {
                            return Err(format!("reel {r} references unknown symbol id {sym}"));
                        }
                        let wv = wmap[&sym];
                        sym_count[r][sym] = wv;
                        cum += wv;
                        wsyms[r].push(sym);
                        wcum[r].push(cum);
                    }
                    reel_size[r] = cum;
                }
            }
            (None, None) => return Err("reels must provide `strips` or `weights`".into()),
        }

        for (r, line) in cfg.paylines.iter().enumerate() {
            if line.len() != reels {
                return Err(format!("payline {r} has {} cells, expected {reels}", line.len()));
            }
            if line.iter().any(|&row| row >= rows) {
                return Err(format!("payline {r} references a row >= rows ({rows})"));
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
            n_symbols,
            max_count,
            is_wild,
            is_scatter,
            pays,
            scatter_id,
            model,
            sym_count,
            reel_size,
            strips,
            columns,
            wsyms,
            wcum,
        })
    }

    /// Sample a symbol on reel `r` from its weight distribution given a uniform draw `x < reel_size`.
    #[inline]
    pub fn sample_weighted(&self, r: usize, x: u64) -> usize {
        let cum = &self.wcum[r];
        // linear scan (few symbols); cum is ascending and last == reel_size
        let mut i = 0;
        while i < cum.len() && x >= cum[i] {
            i += 1;
        }
        self.wsyms[r][i.min(cum.len() - 1)]
    }

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

    fn line_pay_ltr(&self, line: &[usize]) -> u64 {
        if line.is_empty() || self.is_scatter[line[0]] {
            return 0;
        }
        if self.is_wild[line[0]] {
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
            self.run_pay(line[0], line)
        }
    }

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
            LineEval::Ways243 => 0,
        }
    }

    #[inline]
    pub fn scatter_pay(&self, count: usize) -> u64 {
        match self.scatter_id {
            Some(id) => self.pays[id][count.min(self.max_count)],
            None => 0,
        }
    }
}
