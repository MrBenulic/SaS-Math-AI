//! SaS-Math AI — core math engine (M1: 5x3 lines games).
//!
//! Deterministic, integer-accounted slot math. Two cooperating engines on one config:
//! - exact enumeration RTP (line marginals + scatter convolution) — certification truth (ADR-001)
//! - deterministic Monte Carlo over Rayon — validates the harness (ADR-003 namespaced RNG)
//!
//! Money is integer smallest-units throughout (ADR-002). See ../README.md and ../../docs.

pub mod accounting;
pub mod config;
pub mod enumerate;
pub mod mechanics;
pub mod rng;
pub mod simulate;
pub mod solver;
pub mod stats;

pub use config::GameConfig;
pub use enumerate::{exact_rtp, ExactResult};
pub use simulate::{monte_carlo, McResult};
pub use solver::{solve, solve_multi, SolveOptions, SolveResult, SolveTargets};
pub use stats::Accumulator;
