//! The night sky: which constellations are up on a given night.
//!
//! Every night shows [`MAJORS_PER_NIGHT`] of the twenty-one Akstada and
//! [`MINORS_PER_NIGHT`] of the [`MINOR_CONSTELLATIONS`]. The sky turns once a
//! day, at noon, and the new set is the one that rises that night.
//!
//! The sky has its own random stream, seeded from the game's seed and stepped
//! only at noon. So the sky on the `n`th night depends on the seed and on `n`
//! alone: travel, encounters and anything else that draws on the world's
//! [`Rng`] never change it.
//!
//! The rules, from one night to the next:
//!
//! - The first night draws evenly: any four Akstada and any ten minors.
//! - An Akstada that was up last night weighs 50% more in the draw.
//! - Once it has been up [`MAJOR_COOLDOWN_AFTER`] nights running, it weighs
//!   70% less than a returning one would, and half that again for each
//!   further night.
//!   A night away resets it.
//! - A minor that was up last night cannot be up tonight. The rest draw evenly.

use crate::character::Akstada;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// Akstada in the sky each night.
pub const MAJORS_PER_NIGHT: usize = 4;
/// Minor constellations in the sky each night.
pub const MINORS_PER_NIGHT: usize = 10;
/// The hour the sky turns to the coming night's set.
pub const SKY_TURNS_AT_HOUR: u32 = 12;

/// Draw weight of an Akstada that was not up last night. The other weights
/// are in the same units, so 1500 is "50% more likely".
pub const MAJOR_BASE_WEIGHT: u64 = 1000;
/// An Akstada that was up last night: 50% more than the base.
pub const MAJOR_RETURN_WEIGHT: u64 = MAJOR_BASE_WEIGHT * 3 / 2;
/// Nights running after which an Akstada starts to fade.
pub const MAJOR_COOLDOWN_AFTER: u32 = 4;
/// An Akstada up [`MAJOR_COOLDOWN_AFTER`] nights running: 70% less than the
/// return weight. Each night past that halves it again.
///
/// The 70% and each further 50% are taken off the return weight, not off
/// the base, and compound: 450, 225, 112, and so on.
pub const MAJOR_COOLDOWN_WEIGHT: u64 = MAJOR_RETURN_WEIGHT * 3 / 10;

/// Placeholder names for the minor constellations, until the lore has them.
pub const MINOR_CONSTELLATIONS: [&str; 40] = [
    "the Lantern",
    "the Ladle",
    "the Kettle",
    "the Plough",
    "the Spindle",
    "the Loom",
    "the Anvil",
    "the Bellows",
    "the Key",
    "the Lock",
    "the Bridle",
    "the Yoke",
    "the Sickle",
    "the Scythe",
    "the Millstone",
    "the Well",
    "the Ferryman",
    "the Shepherd",
    "the Weaver",
    "the Mason",
    "the Drummer",
    "the Piper",
    "the Twins",
    "the Widow",
    "the Owl",
    "the Heron",
    "the Moth",
    "the Toad",
    "the Boar",
    "the Stag",
    "the Badger",
    "the Serpent",
    "the Oak",
    "the Thorn",
    "the Reed",
    "the Barley",
    "the Crown",
    "the Bell",
    "the Cup",
    "the Ship",
];

/// One of the [`MINOR_CONSTELLATIONS`], by its place in that list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Minor(pub u8);

impl Minor {
    pub fn name(self) -> &'static str {
        MINOR_CONSTELLATIONS[usize::from(self.0)]
    }

    pub fn all() -> impl Iterator<Item = Minor> {
        (0..MINOR_CONSTELLATIONS.len() as u8).map(Minor)
    }
}

/// Mixed into the game's seed so the sky's stream is not the world's.
const SKY_SALT: u64 = 0x0005_EED0_FA65_7ADA;

/// The sky and the memory it needs to draw the next night.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sky {
    rng: Rng,
    /// Nights drawn since the game began. The first night is 0.
    night: u32,
    /// Tonight's Akstada, in the order they were drawn.
    majors: Vec<Akstada>,
    /// Tonight's minor constellations, in the order they were drawn.
    minors: Vec<Minor>,
    /// Nights running each Akstada has been up, indexed like [`Akstada::ALL`].
    /// Zero for one that is down tonight.
    streaks: Vec<u32>,
}

impl Default for Sky {
    /// An empty sky. A world read from a save made before the sky existed
    /// starts here and is filled in on load.
    fn default() -> Sky {
        Sky {
            rng: Rng::new(0),
            night: 0,
            majors: Vec::new(),
            minors: Vec::new(),
            streaks: vec![0; Akstada::ALL.len()],
        }
    }
}

impl Sky {
    /// The first night's sky for a game started from `seed`.
    pub fn new(seed: u64) -> Sky {
        let mut sky = Sky {
            rng: Rng::new(seed ^ SKY_SALT),
            ..Sky::default()
        };
        sky.draw();
        sky
    }

    /// The sky on night `night` of a game started from `seed`, by drawing
    /// every night before it. Night 0 is the first.
    pub fn on_night(seed: u64, night: u32) -> Sky {
        let mut sky = Sky::new(seed);
        for _ in 0..night {
            sky.turn();
        }
        sky
    }

    /// True for a sky that was never drawn.
    pub fn is_empty(&self) -> bool {
        self.majors.is_empty()
    }

    /// Nights drawn since the game began. The first night is 0.
    pub fn night(&self) -> u32 {
        self.night
    }

    /// Tonight's four Akstada.
    pub fn majors(&self) -> &[Akstada] {
        &self.majors
    }

    /// Tonight's ten minor constellations.
    pub fn minors(&self) -> &[Minor] {
        &self.minors
    }

    /// Nights running `akstada` has been up, counting tonight. Zero if it is
    /// down tonight.
    pub fn streak(&self, akstada: Akstada) -> u32 {
        self.streaks[akstada_index(akstada)]
    }

    /// Moves to the next night. The world calls this at noon.
    pub fn turn(&mut self) {
        self.night += 1;
        self.draw();
    }

    /// How heavily `akstada` weighs in tonight's draw for the next night.
    pub fn major_weight(&self, akstada: Akstada) -> u64 {
        weight_for_streak(self.streak(akstada))
    }

    /// Draws tonight's set from last night's, and updates the streaks.
    fn draw(&mut self) {
        let weights: Vec<u64> = Akstada::ALL
            .iter()
            .map(|&a| weight_for_streak(self.streaks[akstada_index(a)]))
            .collect();
        let picked = draw_weighted(&mut self.rng, weights, MAJORS_PER_NIGHT);
        for (i, streak) in self.streaks.iter_mut().enumerate() {
            *streak = if picked.contains(&i) { *streak + 1 } else { 0 };
        }
        self.majors = picked.into_iter().map(|i| Akstada::ALL[i]).collect();

        // A minor up last night has no chance tonight; the rest are even.
        let weights: Vec<u64> = Minor::all()
            .map(|m| if self.minors.contains(&m) { 0 } else { 1 })
            .collect();
        let picked = draw_weighted(&mut self.rng, weights, MINORS_PER_NIGHT);
        self.minors = picked.into_iter().map(|i| Minor(i as u8)).collect();
    }
}

/// Draw weight for an Akstada that has been up `streak` nights running.
pub fn weight_for_streak(streak: u32) -> u64 {
    match streak {
        0 => MAJOR_BASE_WEIGHT,
        s if s < MAJOR_COOLDOWN_AFTER => MAJOR_RETURN_WEIGHT,
        s => {
            let halvings = (s - MAJOR_COOLDOWN_AFTER).min(63);
            // Never quite zero, so a sky can always be filled.
            (MAJOR_COOLDOWN_WEIGHT >> halvings).max(1)
        }
    }
}

/// Picks `count` different indexes, each with chance in proportion to its
/// weight among those not yet picked. Weight zero is never picked.
fn draw_weighted(rng: &mut Rng, mut weights: Vec<u64>, count: usize) -> Vec<usize> {
    let mut picked = Vec::with_capacity(count);
    for _ in 0..count {
        let total: u64 = weights.iter().sum();
        if total == 0 {
            break;
        }
        let mut roll = rng.next_u64() % total;
        let chosen = weights
            .iter()
            .position(|&w| {
                if roll < w {
                    true
                } else {
                    roll -= w;
                    false
                }
            })
            .expect("roll is below the total weight");
        weights[chosen] = 0;
        picked.push(chosen);
    }
    picked
}

fn akstada_index(akstada: Akstada) -> usize {
    Akstada::ALL
        .iter()
        .position(|&a| a == akstada)
        .expect("every Akstada is in Akstada::ALL")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_follow_the_streak() {
        assert_eq!(weight_for_streak(0), 1000);
        assert_eq!(weight_for_streak(1), 1500);
        assert_eq!(weight_for_streak(3), 1500);
        assert_eq!(weight_for_streak(4), 450);
        assert_eq!(weight_for_streak(5), 225);
        assert_eq!(weight_for_streak(6), 112);
        assert_eq!(weight_for_streak(100), 1);
    }

    #[test]
    fn zero_weight_is_never_drawn() {
        let mut rng = Rng::new(3);
        for _ in 0..500 {
            let picked = draw_weighted(&mut rng, vec![0, 1, 0, 1, 1], 3);
            assert!(!picked.contains(&0) && !picked.contains(&2));
        }
    }
}
