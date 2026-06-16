//! Integer / fixed-point money (ADR-002). Money is `u128` smallest-units; RTP is an exact
//! integer ratio rendered to decimal only at the boundary.

/// An exact RTP as an integer ratio `num / den` (kept exact; never summed in float).
#[derive(Debug, Clone, Copy)]
pub struct Rational {
    pub num: u128,
    pub den: u128,
}

impl Rational {
    #[inline]
    pub fn as_f64(&self) -> f64 {
        if self.den == 0 {
            0.0
        } else {
            self.num as f64 / self.den as f64
        }
    }
}

/// RTP as a ratio of integer accumulators (rendered at the boundary).
#[inline]
pub fn rtp_ratio(total_won: u128, total_bet: u128) -> f64 {
    if total_bet == 0 {
        0.0
    } else {
        total_won as f64 / total_bet as f64
    }
}
