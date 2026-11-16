//! The factions a character can belong to, and the hostility they start with.
//!
//! Hostility is directed. The Audsen Line's hostility toward the player is not
//! the player's hostility toward the Line. 100 is the least hostile and 0 is
//! the most hostile. A faction starts at 100 toward itself. The index runs
//! from [`MIN_HOSTILITY`] to [`MAX_HOSTILITY`].
//!
//! [`starting_hostility`](FactionId::starting_hostility) is only the value a
//! new world begins at. The live index is [`crate::World::hostility_toward`],
//! and it changes with the world.

use serde::{Deserialize, Serialize};

/// The most hostile end of the scale.
pub const MIN_HOSTILITY: u32 = 0;
/// The least hostile end of the scale. A faction starts here toward itself.
pub const MAX_HOSTILITY: u32 = 100;

/// One of the factions in the game so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionId {
    Ecumenci,
    Bandit,
    /// The Audsen Line.
    Audsen,
    /// The player's own faction. Followers are given it. It is not joinable:
    /// the player already belongs to it.
    Player,
}

impl FactionId {
    pub const ALL: [FactionId; 4] = [
        FactionId::Ecumenci,
        FactionId::Bandit,
        FactionId::Audsen,
        FactionId::Player,
    ];

    pub fn name(self) -> &'static str {
        match self {
            FactionId::Ecumenci => "Ecumenci",
            FactionId::Bandit => "Bandit",
            FactionId::Audsen => "Audsen Line",
            FactionId::Player => "Player",
        }
    }

    /// True when the player can potentially join this faction.
    ///
    /// The three groups are open. The Player faction is not: a follower is
    /// given it, and the player already has it.
    pub fn joinable(self) -> bool {
        matches!(
            self,
            FactionId::Ecumenci | FactionId::Bandit | FactionId::Audsen
        )
    }

    /// The index this faction starts at toward `other`. A new world copies
    /// these into its own table, which can then change.
    pub fn starting_hostility(self, other: FactionId) -> u32 {
        use FactionId::*;
        match (self, other) {
            (Ecumenci, Ecumenci) | (Bandit, Bandit) | (Audsen, Audsen) | (Player, Player) => {
                MAX_HOSTILITY
            }
            (Ecumenci, Player) => 50,
            (Ecumenci, Bandit) => 0,
            (Ecumenci, Audsen) => 50,
            (Audsen, Player) => 70,
            (Audsen, Bandit) => 0,
            (Audsen, Ecumenci) => 50,
            (Bandit, Player) | (Bandit, Ecumenci) | (Bandit, Audsen) => 0,
            (Player, Ecumenci) | (Player, Bandit) | (Player, Audsen) => 50,
        }
    }

    fn index(self) -> usize {
        match self {
            FactionId::Ecumenci => 0,
            FactionId::Bandit => 1,
            FactionId::Audsen => 2,
            FactionId::Player => 3,
        }
    }
}

/// The live hostility of every faction toward every faction.
///
/// Row is the faction, column is who the index is about, both in
/// [`FactionId::ALL`] order. 100 is least hostile and 0 is most hostile.
/// Saved with the world.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hostility {
    indexes: [[u32; 4]; 4],
}

impl Hostility {
    /// The table a new world starts from.
    pub fn starting() -> Hostility {
        let mut indexes = [[0; 4]; 4];
        for from in FactionId::ALL {
            for toward in FactionId::ALL {
                indexes[from.index()][toward.index()] = from.starting_hostility(toward);
            }
        }
        Hostility { indexes }
    }

    pub fn get(&self, from: FactionId, toward: FactionId) -> u32 {
        self.indexes[from.index()][toward.index()]
    }

    pub fn set(&mut self, from: FactionId, toward: FactionId, index: u32) {
        self.indexes[from.index()][toward.index()] = index;
    }
}

impl Default for Hostility {
    fn default() -> Hostility {
        Hostility::starting()
    }
}
