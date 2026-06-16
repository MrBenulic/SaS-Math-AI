//! Game-config structs — the Rust `serde` target of the tri-target schema (ADR-004).
//! Mirrors `schema/game-config.schema.json` (M1 lines-game subset).

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {
    pub schema_version: String,
    pub game: Game,
    pub symbols: Vec<Symbol>,
    pub reels: Reels,
    pub paytable: Vec<PaytableEntry>,
    pub paylines: Vec<Vec<usize>>,
    pub line_eval: LineEval,
    pub max_win: MaxWin,
    pub rng: RngConfig,
    #[serde(default)]
    pub features: Vec<serde_json::Value>,
    #[serde(default)]
    pub targets: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Game {
    pub name: String,
    pub grid: Grid,
    pub bet_units: u64,
    #[serde(default = "default_denom")]
    pub denomination_scale: u64,
}
fn default_denom() -> u64 {
    100
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grid {
    pub reels: usize,
    pub rows: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Symbol {
    pub id: usize,
    pub name: String,
    pub kind: SymbolKind,
    #[serde(default)]
    pub substitutes_for: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Regular,
    Wild,
    Scatter,
}

/// M1 supports explicit reel strips. (Symbol-weight reels arrive with the solver in M2.)
#[derive(Debug, Clone, Deserialize)]
pub struct Reels {
    pub strips: Vec<Vec<usize>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaytableEntry {
    pub symbol_id: usize,
    /// match-count -> payout in integer smallest-units (JSON string keys parsed to u32).
    pub pays: HashMap<u32, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum LineEval {
    #[serde(rename = "left_to_right")]
    LeftToRight,
    #[serde(rename = "both_ways")]
    BothWays,
    #[serde(rename = "ways_243")]
    Ways243,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct MaxWin {
    pub cap_x: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RngConfig {
    pub generator: String,
    pub master_seed: u64,
}

impl GameConfig {
    pub fn from_json_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}
