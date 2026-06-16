//! Counter-based, namespaced RNG (ADR-003) — the keystone.
//!
//! Every spin's randomness is a pure function of `(master_seed, subsystem, spin_index)`, so:
//! - Rayon thread scheduling never affects outcomes (determinism), and
//! - adding draws in one subsystem never perturbs another's bitstream (stability under code change).
//!
//! We seed a fresh ChaCha8 stream per `(subsystem, spin_index)` via a splitmix64-expanded seed,
//! then draw sequentially within the spin (the `draw_index`). ChaCha8 is a named, cross-platform
//! deterministic primitive with an astronomically large period.

use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

/// RNG subsystems — partition the random domain so features never disturb base reels.
pub const NS_BASE_REELS: u64 = 1;
pub const NS_FEATURE: u64 = 2;
pub const NS_BONUS_PICK: u64 = 3;

#[inline]
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Deterministic 32-byte ChaCha seed from the namespace coordinates (pure integer ops).
fn seed_from(master: u64, subsystem: u64, spin: u64) -> [u8; 32] {
    let mut state = master
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(subsystem.rotate_left(17).wrapping_mul(0xD1B54A32D192ED03))
        .wrapping_add(spin.wrapping_mul(0xFF51AFD7ED558CCD));
    let mut out = [0u8; 32];
    for chunk in out.chunks_mut(8) {
        chunk.copy_from_slice(&splitmix64(&mut state).to_le_bytes());
    }
    out
}

/// A namespaced RNG bound to one `(subsystem, spin_index)` stream.
pub struct NamespacedRng {
    inner: ChaCha8Rng,
}

impl NamespacedRng {
    #[inline]
    pub fn for_spin(master_seed: u64, subsystem: u64, spin_index: u64) -> Self {
        NamespacedRng {
            inner: ChaCha8Rng::from_seed(seed_from(master_seed, subsystem, spin_index)),
        }
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    /// Unbiased uniform integer in `[0, n)` via rejection sampling (no modulo bias).
    #[inline]
    pub fn below(&mut self, n: u32) -> u32 {
        debug_assert!(n > 0);
        let n64 = n as u64;
        let threshold = (1u64 << 32) % n64; // reject the first (2^32 mod n) values
        loop {
            let x = self.inner.next_u32() as u64;
            if x >= threshold {
                return (x % n64) as u32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_coords_same_stream() {
        let mut a = NamespacedRng::for_spin(42, NS_BASE_REELS, 7);
        let mut b = NamespacedRng::for_spin(42, NS_BASE_REELS, 7);
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_spins_differ() {
        let mut a = NamespacedRng::for_spin(42, NS_BASE_REELS, 7);
        let mut b = NamespacedRng::for_spin(42, NS_BASE_REELS, 8);
        let mut diffs = 0;
        for _ in 0..64 {
            if a.next_u32() != b.next_u32() {
                diffs += 1;
            }
        }
        assert!(diffs > 50, "streams should be independent");
    }

    #[test]
    fn below_is_in_range() {
        let mut r = NamespacedRng::for_spin(1, NS_BASE_REELS, 1);
        for _ in 0..10_000 {
            assert!(r.below(7) < 7);
        }
    }
}
