//! A character: who they are, what they can endure, and what they carry.
//!
//! The same sheet is used for the player, for NPCs and for followers. [`Role`]
//! says which. A follower is an NPC with that status set. A character's name
//! is the [`crate::Person`] they belong to. A person named only as cast can
//! still leave the sheet empty.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Youngest age the player can be. An NPC may be younger.
pub const MIN_AGE: u32 = 16;
/// Oldest age the player can be. An NPC may be older.
pub const MAX_AGE: u32 = 40;

/// Health a new character starts with.
pub const BASE_HEALTH: u32 = 100;
/// Fatigue a new character starts with.
pub const BASE_FATIGUE: u32 = 100;
/// Resilience a new character starts with.
pub const BASE_RESILIENCE: u32 = 100;
/// Level a new character starts at.
pub const BASE_LEVEL: u32 = 1;

/// The colour of a person's blood. The Neck sorts people by it, and it is
/// the world's notion of race.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blood {
    /// The human baseline of the Neck.
    Red,
    /// Sky-coloured skin and inverted eyes. The aristocratic "blue blood"
    /// is the metaphor; this is the fact.
    Blue,
    /// The people of Kamlor and the far west.
    Black,
}

impl Blood {
    pub fn name(self) -> &'static str {
        match self {
            Blood::Red => "red",
            Blood::Blue => "blue",
            Blood::Black => "black",
        }
    }
}

/// The people a character comes from.
///
/// These are the nine peoples in the ethnicity notes. Aphlian and Maphlian
/// are spelled as the bible and the dictionary spell them; those two notes
/// shorten the names to Aphian and Maphian, and the Yhlmahni note to Ylhmahi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heritage {
    Aphlian,
    Maphlian,
    Shentzi,
    Ptormic,
    Vhinhili,
    Yhlgori,
    Yhlmahni,
    Theylix,
    Zechi,
}

impl Heritage {
    pub fn name(self) -> &'static str {
        match self {
            Heritage::Aphlian => "Aphlian",
            Heritage::Maphlian => "Maphlian",
            Heritage::Shentzi => "Shentzi",
            Heritage::Ptormic => "Ptormic",
            Heritage::Vhinhili => "Vhinhili",
            Heritage::Yhlgori => "Yhlgori",
            Heritage::Yhlmahni => "Yhlmahni",
            Heritage::Theylix => "Theylix",
            Heritage::Zechi => "Zechi",
        }
    }
}

/// A birth sign. The Akstada is a deck of twenty-one constellation cards,
/// and a person is born under one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Akstada {
    Thief,
    Forgotten,
    Harbinger,
    Raven,
    Crow,
    BloodSage,
    /// The notes also spell this Chizel.
    Chisel,
    Dragon,
    Lamb,
    Wretched,
    Vagybonde,
    Pylgrym,
    Wraith,
    WillowMan,
    Wall,
    Flame,
    Sprait,
    Bastion,
    Acolyte,
    Fox,
    Hare,
}

impl Akstada {
    pub fn name(self) -> &'static str {
        match self {
            Akstada::Thief => "the Thief",
            Akstada::Forgotten => "the Forgotten",
            Akstada::Harbinger => "the Harbinger",
            Akstada::Raven => "the Raven",
            Akstada::Crow => "the Crow",
            Akstada::BloodSage => "the Blood Sage",
            Akstada::Chisel => "the Chisel",
            Akstada::Dragon => "the Dragon",
            Akstada::Lamb => "the Lamb",
            Akstada::Wretched => "the Wretched",
            Akstada::Vagybonde => "the Vagybonde",
            Akstada::Pylgrym => "the Pylgrym",
            Akstada::Wraith => "the Wraith",
            Akstada::WillowMan => "the Willow Man",
            Akstada::Wall => "the Wall",
            Akstada::Flame => "the Flame",
            Akstada::Sprait => "the Sprait",
            Akstada::Bastion => "the Bastion",
            Akstada::Acolyte => "the Acolyte",
            Akstada::Fox => "the Fox",
            Akstada::Hare => "the Hare",
        }
    }
}

/// A tongue the character can speak.
///
/// Vinhili is the mountain dialects; the people who speak them are the
/// Vhinhili. Old Shentz and Shentz are the two ages of the valley's tongue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tongue {
    Aphlian,
    Maphlian,
    Shentz,
    OldShentz,
    Ptormyk,
    Vinhili,
    Yhlgori,
    Ecumene,
    Koloudian,
    Kamloran,
}

impl Tongue {
    pub fn name(self) -> &'static str {
        match self {
            Tongue::Aphlian => "Aphlian",
            Tongue::Maphlian => "Maphlian",
            Tongue::Shentz => "Shentz",
            Tongue::OldShentz => "Old Shentz",
            Tongue::Ptormyk => "Ptormyk",
            Tongue::Vinhili => "Vinhili",
            Tongue::Yhlgori => "Yhlgori",
            Tongue::Ecumene => "Ecumene",
            Tongue::Koloudian => "Koloudian",
            Tongue::Kamloran => "Kamloran",
        }
    }
}

/// Whether the character is the player or an NPC. A follower is an NPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Player,
    /// `follower` is true once this NPC travels in the player's service.
    Npc {
        follower: bool,
    },
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Player => "player",
            Role::Npc { .. } => "npc",
        }
    }

    pub fn is_player(self) -> bool {
        matches!(self, Role::Player)
    }

    pub fn is_follower(self) -> bool {
        matches!(self, Role::Npc { follower: true })
    }
}

/// One stack of a thing the character carries. The `key` is the identity:
/// two stacks with the same key are the same thing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub key: String,
    pub name: String,
    /// How many are carried. A stack of one is a single thing.
    pub count: u32,
}

/// What is wrong with a character sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetError {
    AgeOutOfRange(u32),
    NoTongue,
    DuplicateTongue(Tongue),
    BlankBackground,
    BlankItem,
    EmptyStack(String),
    DuplicateItem(String),
}

impl fmt::Display for SheetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SheetError::AgeOutOfRange(age) => {
                write!(f, "age {age} is outside {MIN_AGE} to {MAX_AGE}")
            }
            SheetError::NoTongue => write!(f, "no tongue is known"),
            SheetError::DuplicateTongue(tongue) => {
                write!(f, "the tongue {} is listed twice", tongue.name())
            }
            SheetError::BlankBackground => write!(f, "the background is blank"),
            SheetError::BlankItem => write!(f, "an item has no key or no name"),
            SheetError::EmptyStack(name) => write!(f, "none of \"{name}\" are carried"),
            SheetError::DuplicateItem(key) => write!(f, "\"{key}\" is carried twice"),
        }
    }
}

/// The stats of one character.
///
/// Health, fatigue, resilience and level start at the `BASE_` values. A data
/// file may set them to something else; omitting them keeps the base.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    /// Player, or an NPC who may also be a follower.
    pub role: Role,
    pub age: u32,
    pub blood: Blood,
    pub heritage: Heritage,
    pub akstada: Akstada,
    /// How much harm the body can still take. A new character starts at
    /// [`BASE_HEALTH`].
    #[serde(default = "base_health")]
    pub health: u32,
    /// How much road the body has left in it. A new character starts at
    /// [`BASE_FATIGUE`]. Nothing spends it yet; nights without camp on the
    /// world are the hook.
    #[serde(default = "base_fatigue")]
    pub fatigue: u32,
    /// How well the character weathers what the road does to them. A new
    /// character starts at [`BASE_RESILIENCE`].
    #[serde(default = "base_resilience")]
    pub resilience: u32,
    /// Tongues the character can speak, with no repeats.
    pub tongues: Vec<Tongue>,
    /// The life before the road. There is no closed list of backgrounds yet,
    /// so this is plain text.
    pub background: String,
    /// The side this character belongs to. Blank when they belong to none.
    /// There is no closed list of factions yet.
    #[serde(default)]
    pub faction: String,
    #[serde(default = "base_level")]
    pub level: u32,
    #[serde(default)]
    pub inventory: Vec<Item>,
}

fn base_health() -> u32 {
    BASE_HEALTH
}
fn base_fatigue() -> u32 {
    BASE_FATIGUE
}
fn base_resilience() -> u32 {
    BASE_RESILIENCE
}
fn base_level() -> u32 {
    BASE_LEVEL
}

impl Character {
    /// A new character at full health, fatigue and resilience, level 1,
    /// carrying nothing.
    pub fn new(
        role: Role,
        age: u32,
        blood: Blood,
        heritage: Heritage,
        akstada: Akstada,
        tongues: Vec<Tongue>,
        background: impl Into<String>,
        faction: impl Into<String>,
    ) -> Result<Character, SheetError> {
        let sheet = Character {
            role,
            age,
            blood,
            heritage,
            akstada,
            health: base_health(),
            fatigue: base_fatigue(),
            resilience: base_resilience(),
            tongues,
            background: background.into(),
            faction: faction.into(),
            level: base_level(),
            inventory: Vec::new(),
        };
        sheet.check()?;
        Ok(sheet)
    }

    /// Reports the first thing wrong with the sheet.
    pub fn check(&self) -> Result<(), SheetError> {
        if self.role.is_player() && !(MIN_AGE..=MAX_AGE).contains(&self.age) {
            return Err(SheetError::AgeOutOfRange(self.age));
        }
        if self.tongues.is_empty() {
            return Err(SheetError::NoTongue);
        }
        let mut seen_tongues: Vec<Tongue> = Vec::new();
        for tongue in &self.tongues {
            if seen_tongues.contains(tongue) {
                return Err(SheetError::DuplicateTongue(*tongue));
            }
            seen_tongues.push(*tongue);
        }
        if self.background.trim().is_empty() {
            return Err(SheetError::BlankBackground);
        }
        let mut seen_items: Vec<&str> = Vec::new();
        for item in &self.inventory {
            if item.key.trim().is_empty() || item.name.trim().is_empty() {
                return Err(SheetError::BlankItem);
            }
            if item.count == 0 {
                return Err(SheetError::EmptyStack(item.name.clone()));
            }
            if seen_items.contains(&item.key.as_str()) {
                return Err(SheetError::DuplicateItem(item.key.clone()));
            }
            seen_items.push(item.key.as_str());
        }
        Ok(())
    }

    /// The faction, or "none" when the character belongs to no side.
    pub fn faction_label(&self) -> &str {
        if self.faction.trim().is_empty() {
            "none"
        } else {
            self.faction.as_str()
        }
    }

    /// The tongues as a single line, in the order they are known.
    pub fn tongue_list(&self) -> String {
        self.tongues
            .iter()
            .map(|tongue| tongue.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Adds `item` to the inventory. A stack already held under the same key
    /// grows by `count`. A count of zero adds nothing.
    pub fn carry(&mut self, item: Item) {
        if item.count == 0 {
            return;
        }
        if let Some(stack) = self.inventory.iter_mut().find(|held| held.key == item.key) {
            stack.count = stack.count.saturating_add(item.count);
        } else {
            self.inventory.push(item);
        }
    }

    /// How many of `key` are carried. Zero when the character has none.
    pub fn holding(&self, key: &str) -> u32 {
        self.inventory
            .iter()
            .find(|item| item.key == key)
            .map_or(0, |item| item.count)
    }
}
