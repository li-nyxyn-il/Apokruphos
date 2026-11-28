//! Scripted dialogue: trees of what an NPC says and what the player may
//! answer, the conditions that gate them, and the actions a choice runs.
//!
//! A [`Condition`] only reads the world. An [`Action`] is stored here as a
//! type; running it belongs to the world's conversation code.

use serde::{Deserialize, Serialize};

use crate::character::Item;
use crate::faction::FactionId;
use crate::ids::PersonId;
use crate::npc::Bucket;
use crate::world::World;

/// A whole conversation an NPC can hold.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueTree {
    /// Tried in order. The first whose condition holds is where the
    /// conversation opens.
    pub entries: Vec<Entry>,
    pub nodes: Vec<Node>,
}

impl DialogueTree {
    /// The node with this key, if the tree has one.
    pub fn node(&self, key: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.key == key)
    }
}

/// A way into the tree: a condition and the node it opens at.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub when: Condition,
    pub node: String,
}

/// One thing the NPC says, and the answers the player may give.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Unique within the tree.
    pub key: String,
    /// What the NPC says on arriving here.
    pub line: String,
    /// The name of one of the person's sprites to show at this node.
    #[serde(default)]
    pub sprite: Option<String>,
    #[serde(default)]
    pub choices: Vec<Choice>,
}

/// One answer the player may give.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    /// What the player says.
    pub text: String,
    /// The choice is offered only while this holds.
    #[serde(default)]
    pub when: Condition,
    #[serde(default)]
    pub actions: Vec<Action>,
    /// The node this leads to. `None` ends the conversation.
    #[serde(default)]
    pub next: Option<String>,
}

/// A test of a number against a bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compare {
    AtLeast(u32),
    AtMost(u32),
    /// Both ends are included.
    Between(u32, u32),
}

impl Compare {
    /// True when `value` passes the test.
    pub fn holds(self, value: u32) -> bool {
        match self {
            Compare::AtLeast(low) => value >= low,
            Compare::AtMost(high) => value <= high,
            Compare::Between(low, high) => (low..=high).contains(&value),
        }
    }
}

/// A question about the world that is either true or false. It never
/// changes the world.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    #[default]
    Always,
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    /// This NPC's attitude toward the player.
    Attitude(Compare),
    /// Their primary faction's hostility index toward the player.
    Hostility(Compare),
    Friendliness(Compare),
    /// One of the buckets open to the NPC.
    Bucket(Bucket),
    FactionHostility {
        from: FactionId,
        toward: FactionId,
        cmp: Compare,
    },
    IsFollower,
    InParty,
    Recruitable,
    /// The player's pack holds at least `count` of the named item.
    PlayerHas {
        item: String,
        count: u32,
    },
    /// The NPC's pack holds at least `count` of the named item.
    NpcHas {
        item: String,
        count: u32,
    },
    /// A flag is set in the world.
    Flag(String),
    /// The world clock is in the hours, ends included. Wraps past midnight
    /// when `from` is after `to`.
    HourBetween {
        from: u32,
        to: u32,
    },
    /// The party stands at the site with this key.
    PartyAtSite(String),
}

/// What a chosen answer does to the world.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Moves the NPC's attitude, keeping it from 0 to 100.
    ChangeAttitude(i32),
    SetAttitude(u32),
    ChangeHostility {
        from: FactionId,
        toward: FactionId,
        by: i32,
    },
    /// The NPC's pack to the player's.
    GiveItem {
        item: String,
        count: u32,
    },
    /// The player's pack to the NPC's.
    TakeItem {
        item: String,
        count: u32,
    },
    /// Made from nothing and given to the player.
    GrantItem(Item),
    /// Takes the NPC into the party, whether or not they are recruitable.
    Recruit,
    SetRecruitable(bool),
    SetFlag(String),
    ClearFlag(String),
    WriteJournal(String),
    /// Ends the conversation in a fight. COMBAT: placeholder until combat exists.
    StartCombat,
}

/// A picture an NPC can be shown with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprite {
    /// "portrait", "hostile" and so on. Unique per person.
    pub name: String,
    /// Relative to the game's assets folder.
    pub path: String,
    /// Set when the image is a sheet of frames.
    #[serde(default)]
    pub frame: Option<Frame>,
}

/// Which frame of a sheet to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub columns: u32,
    pub rows: u32,
    pub index: u32,
}

/// How many of the named item a pack holds, by stack name.
fn held(pack: &[Item], name: &str) -> u32 {
    pack.iter()
        .filter(|item| item.name == name)
        .fold(0u32, |sum, item| sum.saturating_add(item.count))
}

impl Condition {
    /// Whether this holds for the NPC `npc` in `world`.
    pub fn holds(&self, world: &World, npc: PersonId) -> bool {
        let standing = world.standing(npc);
        let person = world.person(npc);
        match self {
            Condition::Always => true,
            Condition::All(all) => all.iter().all(|c| c.holds(world, npc)),
            Condition::Any(any) => any.iter().any(|c| c.holds(world, npc)),
            Condition::Not(inner) => !inner.holds(world, npc),
            Condition::Attitude(cmp) => standing.is_some_and(|s| cmp.holds(s.attitude)),
            Condition::Hostility(cmp) => standing.is_some_and(|s| cmp.holds(s.hostility)),
            Condition::Friendliness(cmp) => standing.is_some_and(|s| cmp.holds(s.friendliness)),
            Condition::Bucket(bucket) => standing.is_some_and(|s| s.buckets().contains(bucket)),
            Condition::FactionHostility { from, toward, cmp } => {
                cmp.holds(world.hostility_toward(*from, *toward))
            }
            Condition::IsFollower => person.is_some_and(|p| p.is_follower()),
            Condition::InParty => person.is_some_and(|p| p.in_party),
            Condition::Recruitable => person.is_some_and(|p| p.recruitable),
            Condition::PlayerHas { item, count } => world
                .people
                .iter()
                .find(|p| p.is_player())
                .and_then(|p| p.sheet.as_ref())
                .is_some_and(|sheet| held(&sheet.inventory, item) >= *count),
            Condition::NpcHas { item, count } => person
                .and_then(|p| p.sheet.as_ref())
                .is_some_and(|sheet| held(&sheet.inventory, item) >= *count),
            Condition::Flag(flag) => world.flags.contains(flag),
            Condition::HourBetween { from, to } => {
                if from <= to {
                    (*from..=*to).contains(&world.hour)
                } else {
                    world.hour >= *from || world.hour <= *to
                }
            }
            Condition::PartyAtSite(key) => world
                .party_site
                .zip(world.place(world.party_at))
                .and_then(|(site, place)| place.site(site))
                .is_some_and(|site| &site.key == key),
        }
    }
}
