//! How an NPC behaves toward the player: whether they will talk, and what
//! can be said to them.
//!
//! Two numbers decide it, both from 0 to 100:
//!
//! - the **hostility index** of the NPC's primary faction toward the player,
//!   where 100 is the least hostile ([`crate::World::hostility_toward`]), and
//! - the NPC's own **attitude** toward the player ([`crate::Role::attitude`]).
//!
//! Their **friendliness** is `floor(sqrt(attitude × hostility))`. Below
//! [`MIN_FRIENDLINESS_TO_TALK`] the NPC is too hostile to talk to, and
//! interacting with them starts combat instead.
//!
//! Each of the three numbers falls in a [`Band`], and each band opens a
//! [`Bucket`] of dialogue lines. A follower also gets [`Bucket::Follower`].
//!
//! COMBAT: combat behaviour belongs here once the combat system exists.

use serde::{Deserialize, Serialize};

/// The lowest friendliness at which an NPC will still talk. Below it,
/// interacting with them starts combat.
pub const MIN_FRIENDLINESS_TO_TALK: u32 = 10;
/// The highest score that still counts as [`Band::Low`].
pub const LOW_BAND_TOP: u32 = 30;
/// The highest score that still counts as [`Band::Average`].
pub const AVERAGE_BAND_TOP: u32 = 70;

/// `floor(sqrt(attitude × hostility))`. With both on the 0 to 100 scale, the
/// result is on it too.
pub fn friendliness(attitude: u32, hostility: u32) -> u32 {
    (attitude * hostility).isqrt()
}

/// Where a score sits on the 0 to 100 scale: low is 0 to 30, average is 31
/// to 70, and high is 71 to 100.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Band {
    Low,
    Average,
    High,
}

impl Band {
    pub fn of(score: u32) -> Band {
        if score <= LOW_BAND_TOP {
            Band::Low
        } else if score <= AVERAGE_BAND_TOP {
            Band::Average
        } else {
            Band::High
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Band::Low => "low",
            Band::Average => "average",
            Band::High => "high",
        }
    }
}

/// A group of dialogue lines, opened by where an NPC's scores sit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Bucket {
    Friendliness(Band),
    Hostility(Band),
    Attitude(Band),
    /// Lines only a follower has.
    Follower,
}

impl Bucket {
    pub fn name(self) -> String {
        match self {
            Bucket::Friendliness(band) => format!("{} friendliness", band.name()),
            Bucket::Hostility(band) => format!("{} hostility index", band.name()),
            Bucket::Attitude(band) => format!("{} attitude", band.name()),
            Bucket::Follower => "follower".to_string(),
        }
    }
}

/// One thing the player can say, and what the NPC answers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueLine {
    /// The bucket that must be open for this line to be offered.
    pub bucket: Bucket,
    /// What the player says.
    pub text: String,
    /// What the NPC says back. Blank for no answer.
    #[serde(default)]
    pub reply: String,
}

/// Where an NPC stands toward the player at this moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    pub attitude: u32,
    /// The hostility index of their primary faction toward the player.
    pub hostility: u32,
    pub friendliness: u32,
    pub follower: bool,
}

impl Standing {
    pub fn new(attitude: u32, hostility: u32, follower: bool) -> Standing {
        Standing {
            attitude,
            hostility,
            friendliness: friendliness(attitude, hostility),
            follower,
        }
    }

    /// True unless they are too hostile to talk to.
    pub fn will_talk(&self) -> bool {
        self.friendliness >= MIN_FRIENDLINESS_TO_TALK
    }

    /// The buckets open to them: one each for friendliness, hostility and
    /// attitude, and the follower bucket for a follower.
    pub fn buckets(&self) -> Vec<Bucket> {
        let mut buckets = vec![
            Bucket::Friendliness(Band::of(self.friendliness)),
            Bucket::Hostility(Band::of(self.hostility)),
            Bucket::Attitude(Band::of(self.attitude)),
        ];
        if self.follower {
            buckets.push(Bucket::Follower);
        }
        buckets
    }
}
