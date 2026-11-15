//! A small seeded random number generator (SplitMix64).
//!
//! The generator lives inside the [`crate::World`] and is saved with it. The
//! same seed and the same commands therefore always produce the same history,
//! which makes bugs reproducible and saves exact. It is written out here, not
//! taken from a library, so that its output can never change under an upgrade.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// True with the given chance out of 100. A chance of 0 is never true and
    /// a chance of 100 or more is always true.
    pub fn chance(&mut self, percent: u32) -> bool {
        (self.next_u64() % 100) < u64::from(percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_numbers() {
        let mut a = Rng::new(192);
        let mut b = Rng::new(192);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn chance_respects_its_bounds() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            assert!(!rng.chance(0));
            assert!(rng.chance(100));
        }
    }
}
