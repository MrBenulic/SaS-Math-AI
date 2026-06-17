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

/// One winning run on a payline, for the traced/replay outcome stream (money in integer units).
/// The pays of all `line_wins` on a line sum to that line's `line_pay`.
#[derive(Debug, Clone, Copy)]
pub struct LineWinDetail {
    pub symbol: usize,
    pub count: usize,
    /// `false` = left-to-right anchor; `true` = right-to-left anchor (the `both_ways` reverse pass).
    pub reversed: bool,
    pub pay: u64,
}

/// One winning symbol in a 243-ways (all-ways) spin: a left-anchored run of `run` reels, each
/// containing `≥1` of the symbol-or-wild; `ways` = product of those per-reel counts; `pay` =
/// `pays[symbol][run] * ways`.
#[derive(Debug, Clone, Copy)]
pub struct WaysWinDetail {
    pub symbol: usize,
    pub run: usize,
    pub ways: u64,
    pub pay: u64,
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
        // The engine is the certification authority and must validate even if an upstream schema/
        // Pydantic check is bypassed (defense in depth).
        if cfg.game.bet_units == 0 {
            return Err("game.bet_units must be >= 1 (it is the RTP denominator)".into());
        }
        if !cfg.max_win.cap_x.is_finite() || cfg.max_win.cap_x <= 0.0 {
            return Err(format!("max_win.cap_x must be finite and > 0, got {}", cfg.max_win.cap_x));
        }
        // Only ChaCha8 is implemented (rng.rs). Reject any other generator rather than silently
        // running ChaCha8 and mislabeling it (the cert artifact must state the RNG that actually ran).
        if cfg.rng.generator != "chacha" {
            return Err(format!(
                "rng.generator '{}' is not supported; only 'chacha' (ChaCha8 counter-RNG) is implemented",
                cfg.rng.generator
            ));
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
                    if scatter_id.is_some() {
                        return Err("multiple scatter symbols are not supported (single-scatter model)".into());
                    }
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
            for (count_str, &payout) in &e.pays {
                let c: usize = count_str
                    .parse()
                    .map_err(|_| format!("paytable count key '{count_str}' is not an integer"))?;
                if c < 1 {
                    return Err(format!("paytable count key '{count_str}' must be >= 1 (no pay for 0 matches)"));
                }
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
                    // parse string keys -> (symbol_id, weight), positive only, sorted by id
                    let mut pairs: Vec<(usize, u64)> = Vec::new();
                    for (k, &wv) in wmap {
                        if wv == 0 {
                            continue;
                        }
                        let sym: usize = k
                            .parse()
                            .map_err(|_| format!("reel {r} weight key '{k}' is not an integer"))?;
                        if sym >= n_symbols {
                            return Err(format!("reel {r} references unknown symbol id {sym}"));
                        }
                        pairs.push((sym, wv));
                    }
                    if pairs.is_empty() {
                        return Err(format!("reel {r} has no positive weights"));
                    }
                    pairs.sort_by_key(|&(s, _)| s);
                    let mut cum = 0u64;
                    for (sym, wv) in pairs {
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

        // Reel sampling draws a stop/cell index as a u32 (rng.below), so a reel's size must fit u32.
        for r in 0..reels {
            if reel_size[r] == 0 {
                return Err(format!("reel {r} has zero size"));
            }
            if reel_size[r] > u32::MAX as u64 {
                return Err(format!(
                    "reel {r} size {} exceeds u32::MAX — reel sampling is 32-bit",
                    reel_size[r]
                ));
            }
        }

        for (r, line) in cfg.paylines.iter().enumerate() {
            if line.len() != reels {
                return Err(format!("payline {r} has {} cells, expected {reels}", line.len()));
            }
            if line.iter().any(|&row| row >= rows) {
                return Err(format!("payline {r} references a row >= rows ({rows})"));
            }
        }
        // Line/both-ways games pay along paylines; an empty payline set would silently yield 0 line RTP.
        if cfg.line_eval != LineEval::Ways243 && cfg.paylines.is_empty() {
            return Err("a line game needs at least one payline".into());
        }

        // cap_x is validated finite & > 0 above; bet_units >= 1. The product is non-negative; the
        // u64::MAX sentinel means "effectively no cap" for astronomically large caps.
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

    /// The single left-anchored winning run on `line`: `(symbol, run length, pay)`; `(0, 0, 0)` if
    /// the line cannot pay left-to-right. A wild leader picks the best-paying non-scatter symbol.
    #[inline]
    fn ltr_run(&self, line: &[usize]) -> (usize, usize, u64) {
        if line.is_empty() || self.is_scatter[line[0]] {
            return (0, 0, 0);
        }
        let pay_of = |sym: usize, c: usize| -> u64 {
            if c >= 1 && c <= self.max_count {
                self.pays[sym][c]
            } else {
                0
            }
        };
        if self.is_wild[line[0]] {
            let mut best = (0usize, 0usize, 0u64);
            for sym in 0..self.n_symbols {
                if self.is_scatter[sym] {
                    continue;
                }
                let c = self.run_len(sym, line);
                let p = pay_of(sym, c);
                if p > best.2 {
                    best = (sym, c, p);
                }
            }
            best
        } else {
            let s = line[0];
            let c = self.run_len(s, line);
            (s, c, pay_of(s, c))
        }
    }

    #[inline]
    fn reversed<'a>(line: &[usize], buf: &'a mut [usize; MAX_REELS]) -> &'a [usize] {
        let n = line.len();
        for i in 0..n {
            buf[i] = line[n - 1 - i];
        }
        &buf[..n]
    }

    #[inline]
    pub fn line_pay(&self, line: &[usize]) -> u64 {
        match self.line_eval {
            LineEval::LeftToRight => self.ltr_run(line).2,
            LineEval::BothWays => {
                let (_, count, pay) = self.ltr_run(line);
                // A run that spans the whole line is the SAME combination from either end — count
                // it once, not once per direction (the old code double-paid full-line wins).
                if count == line.len() {
                    pay
                } else {
                    let mut buf = [0usize; MAX_REELS];
                    pay + self.ltr_run(Self::reversed(line, &mut buf)).2
                }
            }
            LineEval::Ways243 => 0, // ways games evaluate the whole grid via `ways_pay`, not paylines
        }
    }

    /// The winning runs on `line` (for the traced/replay outcome stream). Their pays sum to
    /// [`Self::line_pay`] for the same line.
    pub fn line_wins(&self, line: &[usize]) -> Vec<LineWinDetail> {
        let mut out = Vec::new();
        match self.line_eval {
            LineEval::LeftToRight => {
                let (symbol, count, pay) = self.ltr_run(line);
                if pay > 0 {
                    out.push(LineWinDetail { symbol, count, reversed: false, pay });
                }
            }
            LineEval::BothWays => {
                let (symbol, count, pay) = self.ltr_run(line);
                if pay > 0 {
                    out.push(LineWinDetail { symbol, count, reversed: false, pay });
                }
                // Skip the reverse pass for a full-line win (same combination both ways).
                if count != line.len() {
                    let mut buf = [0usize; MAX_REELS];
                    let (rsym, rcount, rpay) = self.ltr_run(Self::reversed(line, &mut buf));
                    if rpay > 0 {
                        out.push(LineWinDetail { symbol: rsym, count: rcount, reversed: true, pay: rpay });
                    }
                }
            }
            LineEval::Ways243 => {} // ways games use `ways_wins` (whole-grid), not per-payline
        }
        out
    }

    #[inline]
    pub fn scatter_pay(&self, count: usize) -> u64 {
        match self.scatter_id {
            // A pay is only ever earned for >= 1 scatter; never pay on zero (guards a stray "0" key).
            Some(id) if count >= 1 => self.pays[id][count.min(self.max_count)],
            _ => 0,
        }
    }

    /// Number of cells in reel `r`'s visible window that count toward symbol `s` (itself, or — for a
    /// non-wild `s` — a wild). Shared by 243-ways MC and the exact ways math.
    #[inline]
    fn ways_count(&self, grid: &[[usize; MAX_ROWS]; MAX_REELS], r: usize, s: usize) -> u64 {
        let s_is_wild = self.is_wild[s];
        let mut c = 0u64;
        for row in 0..self.rows {
            let cell = grid[r][row];
            let matches = if s_is_wild { cell == s } else { cell == s || self.is_wild[cell] };
            if matches {
                c += 1;
            }
        }
        c
    }

    /// 243-ways (all-ways) total pay for a visible grid: for each non-scatter symbol, the
    /// left-anchored run of reels with `≥1` symbol-or-wild, paying `pays[symbol][run] * (product of
    /// per-reel counts)`. Whole-grid (paylines are not used in ways games).
    pub fn ways_pay(&self, grid: &[[usize; MAX_ROWS]; MAX_REELS]) -> u64 {
        let mut total = 0u64;
        for s in 0..self.n_symbols {
            if self.is_scatter[s] {
                continue;
            }
            let mut ways: u64 = 1;
            let mut run = 0usize;
            for r in 0..self.reels {
                let c = self.ways_count(grid, r, s);
                if c == 0 {
                    break;
                }
                ways = ways.saturating_mul(c);
                run += 1;
            }
            if run >= 1 && run <= self.max_count {
                let pay = self.pays[s][run];
                if pay > 0 {
                    total = total.saturating_add(pay.saturating_mul(ways));
                }
            }
        }
        total
    }

    /// Per-symbol 243-ways wins for the traced/replay outcome stream. Pays sum to [`Self::ways_pay`].
    pub fn ways_wins(&self, grid: &[[usize; MAX_ROWS]; MAX_REELS]) -> Vec<WaysWinDetail> {
        let mut out = Vec::new();
        for s in 0..self.n_symbols {
            if self.is_scatter[s] {
                continue;
            }
            let mut ways: u64 = 1;
            let mut run = 0usize;
            for r in 0..self.reels {
                let c = self.ways_count(grid, r, s);
                if c == 0 {
                    break;
                }
                ways = ways.saturating_mul(c);
                run += 1;
            }
            if run >= 1 && run <= self.max_count {
                let pay = self.pays[s][run].saturating_mul(ways);
                if pay > 0 {
                    out.push(WaysWinDetail { symbol: s, run, ways, pay });
                }
            }
        }
        out
    }

    #[inline]
    pub fn is_ways(&self) -> bool {
        self.line_eval == LineEval::Ways243
    }
}
